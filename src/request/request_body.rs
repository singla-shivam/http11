use crate::helpers::bytes::FragmentedBytes;

#[derive(Debug)]
pub enum RequestBody {
    Whole(FragmentedBytes),
    Chunked(FragmentedBytes),
}
