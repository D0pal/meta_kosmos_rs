use meta_cefi::bitfinex::{common::*, errors::*, events::*, symbol::*, websockets::*};

struct WebSocketHandler;

impl EventHandler for WebSocketHandler {
    fn on_connect(&mut self, event: NotificationEvent) {
        if let NotificationEvent::Info(info) = event {
            println!("Platform status: {:?}, Version {}", info.platform, info.version);
        }
    }

    fn on_auth(&mut self, _event: NotificationEvent) {}

    fn on_subscribed(&mut self, event: NotificationEvent) {
        if let NotificationEvent::TradingSubscribed(msg) = event {
            println!("Subscribed: {:?}", msg);
        } else if let NotificationEvent::CandlesSubscribed(msg) = event {
            println!("Subscribed: {:?}", msg);
        } else if let NotificationEvent::RawBookSubscribed(msg) = event {
            println!("Subscribed: {:?}", msg);
        }
    }

    fn on_data_event(&mut self, event: DataEvent) {
        if let DataEvent::BookTradingSnapshotEvent(channel, book_snapshot) = event {
            println!("book snapshot ({}) {:?}", channel, book_snapshot);
        } else if let DataEvent::BookTradingUpdateEvent(channel, book_update) = event {
            println!(
                "book update ({}) - Price {:?}, Amount: {}, Count: {}",
                channel, book_update.price, book_update.amount, book_update.count
            );
        }
        // ... Add for all events you have subscribed (Trades, Books, ...)
    }

    fn on_error(&mut self, message: Error) {
        println!("{:?}", message);
    }
}

fn main() {
    let mut web_socket: WebSockets = WebSockets::new();

    web_socket.add_event_handler(WebSocketHandler);
    web_socket.connect().unwrap(); // check error

    // BOOKS
    web_socket.subscribe_books(ETHUSD, EventType::Trading, P0, "F0", 25);

    web_socket.event_loop().unwrap(); // check error
}
