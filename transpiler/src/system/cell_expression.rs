use super::{Cell, CellExpression, Column};
use halo2_proofs::pasta::{group::ff::PrimeField, Fp};

pub trait ToCellExpression {
    fn to_cell_expression(self) -> CellExpression;
}

impl ToCellExpression for Cell {
    fn to_cell_expression(self) -> CellExpression {
        CellExpression::CellValue(self)
    }
}

impl ToCellExpression for String {
    fn to_cell_expression(self) -> CellExpression {
        CellExpression::Constant(self)
    }
}

impl ToCellExpression for i64 {
    fn to_cell_expression(self) -> CellExpression {
        CellExpression::Constant(self.to_string())
    }
}

impl ToCellExpression for CellExpression {
    fn to_cell_expression(self) -> CellExpression {
        self
    }
}

impl ToCellExpression for Column {
    fn to_cell_expression(self) -> CellExpression {
        CellExpression::CellValue(self.clone().get_field(0))
    }
}

pub trait ToField {
    // fn to_field<F: PrimeField>(&self) -> Result<F, String>;
    fn to_field<F: PrimeField>(&self) -> Option<F>;
}

impl ToField for String {
    fn to_field<F: PrimeField>(&self) -> Option<F>
    where
        F: PrimeField,
    {
        match self {
            s if s.starts_with("0x") => {
                let mut bytes = F::Repr::default();
                let mut view = bytes.as_mut();
                hex::decode_to_slice(&s[2..], &mut view).ok()?;
                view.reverse();
                // F::from_repr_vartime(bytes).ok_or("Decoding failed".to_string())
                F::from_repr_vartime(bytes)
                // match F::from_repr_vartime(bytes) {
                //     None => panic!("Decoding failed: {}", s),
                //     Some(x) => x,
                // }
            }
            // s => Ok(F::from(s.parse::<u64>().map_err(|e| e.to_string())?)),
            s => Some(F::from(s.parse::<u64>().ok()?)),
        }
    }
}

impl ToString for Fp {
    fn to_string(&self) -> String {
        "0x".to_string()
            + &*self
                .to_repr()
                .as_ref()
                .iter()
                .rev()
                .map(|x| format!("{:02x}", x))
                .collect::<Vec<String>>()
                .join("")
    }
}

#[test]
fn test_fp_to_string() {
    let f = Fp::from(1);
    assert_eq!(
        f.to_string(),
        "0x0000000000000000000000000000000000000000000000000000000000000001".to_string()
    );
}

pub trait ToString {
    fn to_string(&self) -> String;
}
pub trait ToValueString {
    fn to_value_string(&self) -> Option<String>;
}
impl ToField for CellExpression {
    fn to_field<F: PrimeField>(&self) -> Option<F> {
        self.to_value_string().and_then(|x| x.to_field::<F>())
    }
}

impl ToValueString for CellExpression {
    fn to_value_string(&self) -> Option<String> {
        match self {
            CellExpression::Constant(c) => Some(c.clone()),
            CellExpression::CellValue(c) => match c.column.ctype {
                crate::system::ColumnType::Selector => todo!(),
                crate::system::ColumnType::Advice => c.value.clone(),
                crate::system::ColumnType::Fixed => c.value.clone(),
                crate::system::ColumnType::Instance => todo!(),
            },
            CellExpression::Negated(n) => {
                // get_known_value::<Fp>(convert_to_value(*n)?).map(|x| fp_to_string(&(-x)))
                n.to_field::<Fp>().map(|x| (-x).to_string())
            }
            CellExpression::Product(a, b) => match (a.to_field::<Fp>(), b.to_field::<Fp>()) {
                (None, _) | (_, None) => None,
                (Some(a), Some(b)) => Some((a * b).to_string()),
            },
            CellExpression::Sum(a, b) => match (a.to_field::<Fp>(), b.to_field::<Fp>()) {
                (None, _) | (_, None) => None,
                (Some(a), Some(b)) => Some((a + b).to_string()),
            },
            CellExpression::Scaled(a, b) => match (a.to_field::<Fp>(), b.to_field::<Fp>()) {
                (None, _) | (_, None) => None,
                (Some(a), Some(b)) => Some((a * b).to_string()),
            },
        }
    }
}

#[test]
fn test_convert_to_value() {
    let a = "0x0000000000000000000000000000000000000000000000000000000000000002".to_string();
    let b = "0x0000000000000000000000000000000000000000000000000000000000002002".to_string();
    assert_eq!(
        Some("0x0000000000000000000000000000000000000000000000000000000000004004".to_string()),
        (CellExpression::Product(
            Box::new(CellExpression::Constant(a.clone())),
            Box::new(CellExpression::Constant(b.clone())),
        ))
        .to_value_string()
    );
    assert_eq!(
        Some("0x0000000000000000000000000000000000000000000000000000000000002004".to_string()),
        (CellExpression::Sum(
            Box::new(CellExpression::Constant(a.clone())),
            Box::new(CellExpression::Constant(b.clone())),
        ))
        .to_value_string()
    );
    assert_eq!(
        Some("0x0000000000000000000000000000000000000000000000000000000000004004".to_string()),
        (CellExpression::Scaled(Box::new(CellExpression::Constant(a.clone())), b.clone(),))
            .to_value_string()
    );
    assert_eq!(
        Some("0x40000000000000000000000000000000224698fc094cf91b992d30ecffffffff".to_string()),
        (CellExpression::Negated(Box::new(CellExpression::Constant(a.clone())))).to_value_string()
    );
}
