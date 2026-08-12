use crate::base_field_types::Checkbox;
use crate::field_types::{FieldAttributes, FieldType};

impl FieldType for bool {
    type BaseFieldType = Checkbox;
    fn form_field(spec: FieldAttributes, value: &Self) -> Self::BaseFieldType {
        let mut checkbox = Checkbox::new(spec.label);
        if spec.required {
            checkbox = checkbox.required();
        }
        checkbox.checked(*value)
    }

    fn form_extract(field: &Self::BaseFieldType) -> Result<Self, String> {
        Ok(field.is_checked())
    }
}
