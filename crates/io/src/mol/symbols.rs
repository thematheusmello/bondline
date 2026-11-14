//! Shared IUPAC catalog; retain the existing format adapter's Option/error boundary.
use bondline_model::Element;

pub(crate) fn element(symbol: &str) -> Option<Element> {
    Element::from_symbol(symbol)
}
pub(crate) fn symbol(element: Element) -> Option<&'static str> {
    element.symbol()
}
