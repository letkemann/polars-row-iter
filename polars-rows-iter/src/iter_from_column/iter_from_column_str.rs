use super::*;
use iter_from_column_trait::IterFromColumn;
use polars::prelude::*;

impl<'a> IterFromColumn<'a> for &'a str {
    type RawInner = &'a str;
    fn create_iter(column: &'a Column) -> PolarsResult<Box<dyn Iterator<Item = Option<&'a str>> + 'a>> {
        create_iter(column)
    }

    #[inline]
    fn get_value(polars_value: Option<&'a str>, column_name: &str, _dtype: &DataType) -> PolarsResult<Self>
    where
        Self: Sized,
    {
        polars_value.ok_or_else(|| <&'a str as IterFromColumn<'a>>::unexpected_null_value_error(column_name))
    }
}

impl<'a> IterFromColumn<'a> for Option<&'a str> {
    type RawInner = &'a str;
    fn create_iter(column: &'a Column) -> PolarsResult<Box<dyn Iterator<Item = Option<&'a str>> + 'a>> {
        create_iter(column)
    }

    #[inline]
    fn get_value(polars_value: Option<&'a str>, _column_name: &str, _dtype: &DataType) -> PolarsResult<Self>
    where
        Self: Sized,
    {
        Ok(polars_value)
    }
}

fn create_str_iter<'a>(column: &'a Column) -> PolarsResult<Box<dyn Iterator<Item = Option<&'a str>> + 'a>> {
    let inner = column.str()?.iter();
    Ok(Box::new(inner))
}

#[cfg(feature = "dtype-categorical")]
fn create_cat_iter<'a>(column: &'a Column) -> PolarsResult<Box<dyn Iterator<Item = Option<&'a str>> + 'a>> {
    let inner = column.categorical()?.iter_str();
    Ok(Box::new(inner))
}

fn create_iter<'a>(column: &'a Column) -> PolarsResult<Box<dyn Iterator<Item = Option<&'a str>> + 'a>> {
    let iter = match column.dtype() {
        DataType::String => create_str_iter(column)?,
        #[cfg(feature = "dtype-categorical")]
        DataType::Categorical(_, _) => create_cat_iter(column)?,
        dtype => {
            let column_name = column.name().as_str();
            return Err(
                polars_err!(SchemaMismatch: "Cannot get &str from column '{column_name}' with dtype : {dtype}"),
            );
        }
    };

    Ok(iter)
}

#[cfg(test)]
mod tests {
    use crate::*;
    use itertools::{izip, Itertools};
    use polars::prelude::*;
    use rand::{rngs::StdRng, SeedableRng};
    use shared_test_helpers::*;

    const ROW_COUNT: usize = 64;

    #[test]
    fn str_test() {
        let mut rng = StdRng::seed_from_u64(0);
        let height = ROW_COUNT;
        let dtype = DataType::String;

        let col = create_column("col", dtype.clone(), false, height, &mut rng);
        let col_opt = create_column("col_opt", dtype, true, height, &mut rng);

        let col_values = col.str().unwrap().iter().map(|v| v.unwrap().to_owned()).collect_vec();
        let col_opt_values = col_opt
            .str()
            .unwrap()
            .iter()
            .map(|v| v.map(|s| s.to_owned()))
            .collect_vec();

        let df = DataFrame::new(vec![col, col_opt]).unwrap();

        let col_iter = col_values.iter();
        let col_opt_iter = col_opt_values.iter();

        let expected_rows = izip!(col_iter, col_opt_iter)
            .map(|(col, col_opt)| TestRow {
                col: col.as_ref(),
                col_opt: col_opt.as_ref().map(|v| v.as_str()),
            })
            .collect_vec();

        #[derive(Debug, FromDataFrameRow, PartialEq)]
        struct TestRow<'a> {
            col: &'a str,
            col_opt: Option<&'a str>,
        }

        let rows = df.rows_iter::<TestRow>().unwrap().map(|v| v.unwrap()).collect_vec();

        assert_eq!(rows, expected_rows)
    }

    #[cfg(feature = "dtype-categorical")]
    #[test]
    fn cat_test() {
        let mut rng = StdRng::seed_from_u64(0);
        let height = ROW_COUNT;
        let dtype = DataType::Categorical(None, CategoricalOrdering::Physical);

        let col = create_column("col", dtype.clone(), false, height, &mut rng);
        let col_opt = create_column("col_opt", dtype, true, height, &mut rng);

        let col_values = col
            .categorical()
            .unwrap()
            .iter_str()
            .map(|v| v.unwrap().to_owned())
            .collect_vec();
        let col_opt_values = col_opt
            .categorical()
            .unwrap()
            .iter_str()
            .map(|v| v.map(|s| s.to_owned()))
            .collect_vec();

        let df = DataFrame::new(vec![col, col_opt]).unwrap();

        let col_iter = col_values.iter();
        let col_opt_iter = col_opt_values.iter();

        let expected_rows = izip!(col_iter, col_opt_iter)
            .map(|(col, col_opt)| TestRow {
                col: col.as_ref(),
                col_opt: col_opt.as_ref().map(|v| v.as_str()),
            })
            .collect_vec();

        #[derive(Debug, FromDataFrameRow, PartialEq)]
        struct TestRow<'a> {
            col: &'a str,
            col_opt: Option<&'a str>,
        }

        let rows = df.rows_iter::<TestRow>().unwrap().map(|v| v.unwrap()).collect_vec();

        assert_eq!(rows, expected_rows)
    }
}
