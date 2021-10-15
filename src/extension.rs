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
