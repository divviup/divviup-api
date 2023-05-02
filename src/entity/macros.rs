#[macro_export]
macro_rules! json_newtype {
    ($type:ty) => {
        json_newtype!($type where);
    };

    ($type:ty where $($generic_type_name:ident $(: $generic_type_bound:ident $(+ $generic_type_bound2:ident)*)?),*) => {
        impl<$($generic_type_name $(: $generic_type_bound $(+ $generic_type_bound2)*)?),*> From<$type> for sea_orm::Value {
            fn from(value: $type) -> Self {
                sea_orm::Value::Json(serde_json::to_value(&value).ok().map(Box::new))
            }
        }

        impl<$($generic_type_name $(: $generic_type_bound $(+ $generic_type_bound2)*)?),*> sea_orm::TryGetable for $type {
            fn try_get_by<I: sea_orm::ColIdx>(res: &sea_orm::QueryResult, idx: I) -> Result<Self, sea_orm::TryGetError> {
                match res.try_get_by(idx).map_err(sea_orm::TryGetError::DbErr)? {
                    serde_json::Value::Null => Err(sea_orm::TryGetError::Null(format!("{idx:?}"))),
                    json => serde_json::from_value(json).map_err(|e| sea_orm::TryGetError::DbErr(sea_orm::DbErr::Json(e.to_string())))
                }
            }
        }

        impl<$($generic_type_name $(: $generic_type_bound $(+ $generic_type_bound2)*)?),*> sea_orm::sea_query::ValueType for $type {
            fn try_from(v: sea_orm::Value) -> Result<Self, sea_orm::sea_query::ValueTypeErr> {
                match v {
                    sea_orm::Value::Json(Some(x)) => serde_json::from_value(*x).map_err(|_| sea_orm::sea_query::ValueTypeErr),
                    _ => Err(sea_orm::sea_query::ValueTypeErr),
                }
            }

            fn type_name() -> String {
                stringify!($type).to_owned()
            }

            fn array_type() -> sea_orm::sea_query::ArrayType {
                sea_orm::sea_query::ArrayType::Json
            }

            fn column_type() -> sea_orm::entity::ColumnType {
                sea_orm::entity::ColumnType::Json
            }
        }

        impl<$($generic_type_name $(: $generic_type_bound $(+ $generic_type_bound2)*)?),*> sea_orm::sea_query::Nullable for $type {
            fn null() -> sea_orm::Value {
                sea_orm::Value::Json(Some(Box::new(serde_json::Value::Null)))
            }
        }
    };

}
