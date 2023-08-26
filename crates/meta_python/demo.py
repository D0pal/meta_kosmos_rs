import asyncio
import meta_python

ohio_simulator = meta_python.OhioWrapperPy("wss://wss.mantle.xyz")

async def replay_success_tx_async():
    await ohio_simulator.async_init()
    ret = await ohio_simulator.replay_transaction("0xda4a850083e5c12aa39d83952aea08d45f6ec8afcfa295f3dd9f17eaa441aecf")
    print("async result is" ,ret.status, ret.message, ret.transaction_revert_message, ret.gas_used)

async def replay_revert_tx_async():
    await ohio_simulator.async_init()
    ret = await ohio_simulator.replay_transaction("0x2df40ecffc5c3819bce29da319213efda1e0be2bf04143f808fbf3bbfcf9d22e")
    print("async result is" ,ret.status, ret.message, ret.transaction_revert_message, ret.gas_used)

asyncio.run(replay_success_tx_async())

