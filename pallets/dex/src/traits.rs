//swap token
use primitives::{AccountId, Balance, CurrencyId};
use sp_runtime::{DispatchError, DispatchResult};
use support::SwapLimit;

pub trait Exchange<AccountId, Balance, CurrencyId> {
	fn exchange_with_different_currency(
		who: &AccountId,
		path: &[CurrencyId],
		limit: SwapLimit<Balance>,
	) -> Result<(Balance, Balance), DispatchError>;
}
