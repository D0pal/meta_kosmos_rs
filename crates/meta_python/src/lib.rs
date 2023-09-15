use meta_alchemy::{EvmSimulator, ReplayTransactionResult};
use meta_util::ether::tx_hash_from_str;
use pyo3::prelude::*;
use std::{borrow::BorrowMut, sync::Arc};
use tokio::sync::Mutex;

unsafe impl Send for RustBackend {}
pub struct RustBackend {
    pub ws_url: String,
    pub evm_simulator: Option<EvmSimulator>,
}

static STATUS_OK: &str = "OK";
static STATUS_FAILED: &str = "FAILED";

#[derive(Debug)]
#[pyclass]
pub struct ReplayTransactionResponse {
    #[pyo3(get)]
    pub status: String,
    #[pyo3(get)]
    pub message: String,
    #[pyo3(get)]
    pub transaction_revert_message: String,
    #[pyo3(get)]
    pub gas_used: u64,
}

impl Default for ReplayTransactionResponse {
    fn default() -> Self {
        ReplayTransactionResponse {
            status: STATUS_FAILED.to_string(),
            message: "".to_string(),
            transaction_revert_message: "".to_string(),
            gas_used: 0,
        }
    }
}

impl RustBackend {
    pub fn new(url: String) -> Self {
        Self { ws_url: url, evm_simulator: None }
    }

    pub async fn init(&mut self) {
        let simulator = EvmSimulator::new(&self.ws_url, None).await;
        self.evm_simulator = Some(simulator);
    }

    pub async fn replay_transaction(&self, hash: String) -> ReplayTransactionResponse {
        let tx_hash = tx_hash_from_str(&hash);
        if let Some(ref simulator) = self.evm_simulator {
            let ret = simulator.replay_transaction(tx_hash).await;
            match ret {
                Ok(ReplayTransactionResult::Success { gas_used, gas_refunded, output }) => {
                    return ReplayTransactionResponse {
                        status: STATUS_OK.to_string(),
                        message: "".to_string(),
                        transaction_revert_message: "".to_string(),
                        gas_used: gas_used,
                    }
                }
                Ok(ReplayTransactionResult::Revert { gas_used, message }) => {
                    return ReplayTransactionResponse {
                        status: STATUS_OK.to_string(),
                        message: "".to_string(),
                        transaction_revert_message: (&message).to_string(),
                        gas_used: gas_used,
                    }
                }
                _ => return ReplayTransactionResponse::default(),
            }
        }
        return ReplayTransactionResponse::default();
    }
}

#[pymodule]
fn meta_python(_py: Python, m: &PyModule) -> PyResult<()> {
    #[pyclass]
    struct OhioWrapperPy {
        inner: Arc<Mutex<RustBackend>>,
    };

    #[pymethods]
    impl OhioWrapperPy {
        #[new]
        fn new(ws_url: String) -> Self {
            OhioWrapperPy { inner: Arc::new(Mutex::new(RustBackend::new(ws_url))) }
        }
        fn async_init<'p>(&mut self, py: Python<'p>) -> PyResult<&'p PyAny> {
            let backend = self.inner.clone();
            pyo3_asyncio::tokio::future_into_py(py, async move {
                let mut _guard = backend.lock().await;
                _guard.init().await;
                Ok(())
            })
        }

        fn replay_transaction<'p>(&mut self, py: Python<'p>, hash: String) -> PyResult<&'p PyAny> {
            let backend = self.inner.clone();
            let ret = pyo3_asyncio::tokio::future_into_py(py, async move {
                let mut _guard = backend.lock().await;
                let out =_guard.replay_transaction(hash).await;
                Ok(out)
            });
            ret
        }
    }

    m.add_class::<OhioWrapperPy>()?;
    Ok(())
}

