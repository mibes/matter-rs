use crate::{data_model::objects::AttrValue, error::Error};

pub trait AttributeChangeConsumer {
    fn attribute_changed(&mut self, attr_id: u16, value: &AttrValue) -> Result<(), Error>;
}

pub trait AttributeChangeProvider {
    fn register_attribute_change_consumer(&mut self, consumer: Box<dyn AttributeChangeConsumer>);
}
