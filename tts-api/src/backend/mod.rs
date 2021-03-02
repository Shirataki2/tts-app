pub mod openjtalk;

pub trait TtsEngine
where
    Self: Sized,
{
    type Config;
    type Error;

    fn from_config(config: Self::Config) -> Result<Self, Self::Error>;
    fn generate(&self, text: &str) -> Result<Vec<u8>, Self::Error>;
    fn generate_i16(&self, _text: &str) -> Result<Vec<i16>, Self::Error> {
        unimplemented!()
    }
}
