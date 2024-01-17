use super::{SendTransaction, ReceiveTransaction};

pub enum FileTransaction
{
    SendTransaction(SendTransaction),
    ReceiveTransaction(ReceiveTransaction),
}