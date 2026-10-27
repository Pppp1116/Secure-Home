use crate::connection::SendingOrder;

fn do_order(order: &SendingOrder) {
    match order {
        SendingOrder::HumanPresence => {}
    }
}
