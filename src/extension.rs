use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
pub struct Trait {
    pub display_type: Option<String>,
    pub trait_type: String,
    pub value: String,
}

pub trait MetaDataPersonalization {
    fn get_decision_trait(&self, trait_type: &str) -> Option<Trait>;
    fn set_personalized_trait(&mut self, trait_type: &str, value: &str);
    fn set_status(&mut self, status: &str);
    fn get_status(&self) -> Option<String>;
    fn get_token_uri(&self) -> String;
    fn get_image(&self, prefix: &str) -> Option<String>;
    fn get_image_raw(&self) -> Option<String>;
    fn set_image(&mut self, image: Option<String>);
    fn get_name(&self) -> Option<String>;
    fn set_name(&mut self, image: Option<String>);
    fn get_description(&self) -> Option<String>;
    fn set_description(&mut self, image: Option<String>);
}

pub trait MetaPersonalize {
    fn perform_mint(&self, mint_meta: &mut dyn MetaDataPersonalization) -> Option<String>;
}
// see: https://docs.opensea.io/docs/metadata-standards
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
pub struct Metadata {
    pub token_uri: String,
    pub image: Option<String>,
    pub image_data: Option<String>,
    pub external_url: Option<String>,
    pub description: Option<String>,
    pub name: Option<String>,
    pub attributes: Option<Vec<Trait>>,
    pub background_color: Option<String>,
    pub animation_url: Option<String>,
    pub youtube_url: Option<String>,
    pub current_status: Option<String>,
}

impl MetaDataPersonalization for Metadata {
    fn get_decision_trait(&self, trait_type: &str) -> Option<Trait> {
        if let Some(attr) = &self.attributes {
            let trait_attribute = attr
                .iter()
                .filter(|t| t.trait_type == trait_type)
                .cloned()
                .collect::<Vec<_>>();
            trait_attribute.first().cloned()
        } else {
            None
        }
    }
    fn get_token_uri(&self) -> String {
        self.token_uri.clone()
    }

    fn set_personalized_trait(&mut self, trait_type: &str, value: &str) {
        if let Some(attr_list) = &self.attributes {
            let mut new_attr: Vec<Trait> = Default::default();
            for att in attr_list {
                if att.trait_type == trait_type {
                    new_attr.push(Trait {
                        display_type: att.display_type.clone(),
                        trait_type: att.trait_type.clone(),
                        value: value.into(),
                    })
                } else {
                    new_attr.push(att.clone());
                }
            }
            self.name = Some(String::from(value));
            //  self.attributes = Some(new_attr);
            self.attributes = Some(new_attr);
        }
    }
    fn set_status(&mut self, status: &str) {
        self.current_status = Some(String::from(status))
    }
    fn get_status(&self) -> Option<String> {
        self.current_status.clone()
    }
    fn get_image(&self, prefix: &str) -> Option<String> {
        return self.image.as_ref().map(|i| {
            if i.starts_with("ipfs://") || i.starts_with("http") {
                i.clone()
            } else {
                format!("{}{}", prefix, i)
            }
        });
    }
    fn set_image(&mut self, image: Option<String>) {
        self.image = image
    }
    fn get_image_raw(&self) -> Option<String> {
        self.image.clone()
    }
    fn get_name(&self) -> Option<String> {
        self.name.clone()
    }
    fn set_name(&mut self, name: Option<String>) {
        self.set_personalized_trait("name", name.as_ref().unwrap_or(&"".to_string()));
        // self.name = name
    }
    fn get_description(&self) -> Option<String> {
        self.description.clone()
    }
    fn set_description(&mut self, description: Option<String>) {
        self.description = description
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
pub struct BuyMetaData {
    pub male_name: String,
    pub female_name: String,
}
impl MetaPersonalize for BuyMetaData {
    fn perform_mint(&self, mint_meta: &mut dyn MetaDataPersonalization) -> Option<String> {
        if let Some(gender) = mint_meta.get_decision_trait("gender") {
            if gender.value == "male" {
                mint_meta.set_personalized_trait("name", &self.male_name);
                Some(self.male_name.clone())
            } else {
                mint_meta.set_personalized_trait("name", &self.female_name);
                Some(self.female_name.clone())
            }
        } else {
            None
        }
    }
}
