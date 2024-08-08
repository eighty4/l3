use crate::code::source::path::SourcePath;
use crate::code::source::Language;
use crate::lambda::RouteKey;

pub struct SourceUpdate {
    pub kind: SourceUpdateKind,
    pub path: SourcePath,
}

pub enum SourceUpdateKind {
    #[allow(unused)]
    AddLambda(RouteKey),
    #[allow(unused)]
    RemoveLambda(RouteKey),
    #[allow(unused)]
    UpdateCode(RouteKey),
    #[allow(unused)]
    UpdateDependencies(Language),
    #[allow(unused)]
    UpdateEnvironment(RouteKey),
}
