use aws_sdk_lambda::types::Runtime;

pub enum NodeVersion {
    Eighteen,
    Twenty,
    #[allow(unused)]
    TwentyTwo,
}

pub enum PythonVersion {
    #[allow(unused)]
    Three8,
    Three9,
    Three10,
    Three11,
    #[allow(unused)]
    Three12,
    #[allow(unused)]
    Three13,
}

pub enum AwsLambdaRuntime {
    NodeJS(NodeVersion),
    Python(PythonVersion),
    #[allow(unused)]
    Unsupported(Runtime),
}

impl AwsLambdaRuntime {
    /// Returns true for the most up-to-date Node and Python versions on Amazon Linux 2023
    #[allow(unused)]
    pub fn is_most_up_to_date(&self) -> bool {
        match self {
            AwsLambdaRuntime::NodeJS(v) => {
                matches!(v, NodeVersion::Twenty | NodeVersion::TwentyTwo)
            }
            AwsLambdaRuntime::Python(v) => {
                matches!(v, PythonVersion::Three12 | PythonVersion::Three13)
            }
            _ => panic!(),
        }
    }
}

impl From<Runtime> for AwsLambdaRuntime {
    fn from(v: Runtime) -> Self {
        match v {
            Runtime::Nodejs20x => AwsLambdaRuntime::NodeJS(NodeVersion::Twenty),
            Runtime::Nodejs18x => AwsLambdaRuntime::NodeJS(NodeVersion::Eighteen),
            Runtime::Python312 => AwsLambdaRuntime::Python(PythonVersion::Three12),
            Runtime::Python311 => AwsLambdaRuntime::Python(PythonVersion::Three11),
            Runtime::Python310 => AwsLambdaRuntime::Python(PythonVersion::Three10),
            Runtime::Python39 => AwsLambdaRuntime::Python(PythonVersion::Three9),
            _ => AwsLambdaRuntime::Unsupported(v),
        }
    }
}
