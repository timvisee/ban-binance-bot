use futures::{future::Future, prelude::*};

/// Same as `select_ok`, but for booleans.
///
/// This returns with `true` as soon as a future in `iter` yields `true`.
/// If all yield `false`, `false` is returned instead.
///
/// See: `futures::future::select_ok`
pub async fn select_true<I>(iter: I) -> bool
    where I: IntoIterator,
          I::Item: Future<Output = bool> + Send,
{
    // Collect the list, ensure there's at least one future to complete
    let list: Vec<_> = iter.into_iter().collect();
    if list.is_empty() {
        return false;
    }

    futures::future::select_ok(
        list.into_iter().map(|f| btr(f).boxed()),
    ).await.is_ok()
}

/// Convert a `bool` future to an empty `Result` future.
///
/// Function name stands for: Boolean To Result
///
/// This project commonly uses booleans as future return type.
/// Some functions, such as `select_ok` require the future to return a `Result` instead.
///
/// Converts:
/// - `true` to `Ok(())`
/// - `false` to `Err(())`
pub async fn btr<F>(future: F) -> Result<(), ()>
where F: Future<Output = bool> {
    if future.await {
        Ok(())
    } else {
        Err(())
    }
}
