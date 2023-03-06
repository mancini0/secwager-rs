use std::{collections::{BTreeMap, HashMap}, rc::Rc, u8};
use std::vec;

#[derive(Debug)]
struct OrderBook<'a> {
    bids: BTreeMap<u16, Vec<&'a str>>,
    asks: BTreeMap<u16, Vec<&'a str>>,
    arena: HashMap<&'a str, Order<'a>>,
    max_bid: u16,
    min_ask: u16,
}

#[derive(Debug)]
struct Order<'a> {
    id: String,
    order_type: OrderType,
    price: u16,
    qty_open: u16,
    qty_filled:u16,
    symbol: String,
    fill_history: Vec<Fill<'a>>,
    state: OrderState
}

#[derive(Debug)]
struct Fill<'a> {
    price: u16,
    qty: u16,
    filled_against: &'a str
}
#[derive(Debug)]
enum OrderType {
    Buy,
    Sell,
    Cancel,
}

#[derive(Debug, Copy, Clone)]
enum OrderState {
    Open,
    Filled,
    Cancelled,
}

enum MarketSide{
    Buy,
    Sell
}

enum CallbackAction<'a>{
    Publish{id: &'a str},
    PopResting{id: &'a str, side:MarketSide, price: u16}
        
}

impl<'a> OrderBook<'a> {
    pub fn new() -> Self {
        OrderBook {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            arena: HashMap::new(),
            max_bid: 0,
            min_ask: 0
        }
    }
    

    fn handle_buy(&mut self,  incoming_buy:  &mut Order,  callbacks: &mut Vec<CallbackAction> ) {
        'levels: for (price,  resting_orders) in self.asks.range_mut(..incoming_buy.price){
             for i in 0..resting_orders.len(){
                if let Some(resting_sell) = self.arena.get_mut(resting_orders.get(i).unwrap()){
                    let trade_qty = std::cmp::min(incoming_buy.qty_open, resting_sell.qty_open);
                    incoming_buy.qty_open -= trade_qty;
                    incoming_buy.qty_filled += trade_qty;
                   
                    resting_sell.qty_open -= trade_qty;
                    resting_sell.qty_filled+=trade_qty;
                    
                    incoming_buy.fill_history.push(Fill{qty: trade_qty, price: *price, filled_against: &resting_sell.id});
                    resting_sell.fill_history.push(Fill{qty: trade_qty, price: *price, filled_against: &incoming_buy.id});
                    
                    callbacks.push(CallbackAction::Publish{id: &resting_sell.id});
                    callbacks.push(CallbackAction::Publish{id: &incoming_buy.id});

                    if resting_sell.qty_open==0 {
                        resting_sell.state=OrderState::Filled;
                        callbacks.push(CallbackAction::PopResting { id: &resting_sell.id, side: MarketSide::Sell, price: *price })

                    }
                    if incoming_buy.qty_open==0 {
                        incoming_buy.state=OrderState::Filled;
                        break 'levels;
                    }
                }    

            }

        }
        
    } 
        
    



    
    pub fn submit(&mut self, mut order:  Order) {
        match order.order_type {
            OrderType::Buy => Self::handle_buy(self, &mut order, vec![]),
            OrderType::Sell => Self::handle_sell(self, &mut order),
            OrderType::Cancel => Self::handle_cancel(self, &mut order),
        }
    }
    
}
