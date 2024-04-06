use error_set::error_set;


error_set! {
    ActivateError = UnrecoverableError || ActivateEnvError;
    ActivateEnvError = {
        NoEnvironmentsFound,
    } || DeActivateOldEnv || AddEnvError || AddLinkError;
    DeActivateOldEnv = RemoveEnvError || RemoveLinkError;
    RemoveEnvError = {
        IoError(std::io::Error),
        ParsingOldEnv(serde_json::error::Error),
    };
    AddEnvError = {
        WritingToEnvFile(std::io::Error),
    };
    RemoveLinkError = {  
        NotASymlink(UserDisplayError),
        RemoveFile(std::io::Error),
    };
    AddLinkError = {
        CouldNotCreateSymlink(std::io::Error)
    } || UnrecoverableError;
    UnrecoverableError = {
        UserDisplay(UserDisplayError)
    };
}

#[derive(Debug)]
pub(crate) struct UserDisplayError(pub String);

impl std::error::Error for UserDisplayError {}

impl std::fmt::Display for UserDisplayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0);
        Ok(())
    }
}

impl From<String> for UserDisplayError {
    fn from(value: String) -> Self {
        UserDisplayError(value)
    }
}

impl From<&str> for UserDisplayError {
    fn from(value: &str) -> Self {
        UserDisplayError(value.to_owned())
    }
}