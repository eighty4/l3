use crate::code::checksum::ChecksumTree;
use crate::code::source::tracker::SourceUpdate;
use tokio::sync::mpsc::Sender;

#[allow(unused)]
pub struct SourceTree {
    #[allow(unused)]
    checksums: ChecksumTree,
    #[allow(unused)]
    update_sender: Option<Sender<SourceUpdate>>,
}
