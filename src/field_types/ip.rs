use super::text::text_input;
use crate::base_field_types::TextInput;
use crate::field_types::{FieldAttributes, FieldType};

macro_rules! impl_ip_form_value {
    ($($ty:ty => $rule:expr, $message:literal;)*) => {
        $(
            impl FieldType for $ty {
                type BaseFieldType = TextInput;
                fn form_field(spec: FieldAttributes, value: &Self) -> Self::BaseFieldType {
                    text_input(spec, value.to_string(), Some(Box::new($rule))).required()
                }

                fn form_extract(field: &Self::BaseFieldType) -> Result<Self, String> {
                    field.value().parse::<Self>().map_err(|_| $message.to_string())
                }
            }
        )*
    };
}

impl_ip_form_value!(
    std::net::Ipv4Addr => crate::validation::ip::Ipv4, "expected an IPv4 address";
    std::net::Ipv6Addr => crate::validation::ip::Ipv6, "expected an IPv6 address";
);
