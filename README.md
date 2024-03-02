# RcOCell
Wrapper for `Rc<RefCell<Option<T>>>` and its weak variant. 

Includes various utilities for common operations usually performed on such a datastructure.


## Simple Example
```rust
pub fn main() {
    let cell : RcOCell<u8> = RcOCell::from_value(1u8);
    
    //The usual RefCell stuff...
    let borrowed : Ref<u8> = cell.borrow();
    //...
    drop(borrowed);
    let borrowed_mut : RefMut<u8> = cell.borrow_mut();
    //...
    drop(borrowed_mut);
    
   
}
```


## Info
Following functions are provided as part of RcoCell and WeakRcoCell:

In general all functions prefixed "try" will not panic. 
All functions without this prefix may panic depending on what they do.

#### Borrowing: 
* `borrow` and `borrow_mut` 
  * just like `RefCell`
* `try_borrow` and `try_borrow_mut` 
  * just like `RefCell` but error type is an enum.

Those calls will either panic or fail if the cell is empty.
The normal rust borrowing rules apply: only 1 mutable borrow or n normal borrows.
If the borrowing rules are violated at runtime then either panic or failure occurs.

#### Modifying:
* `set` and `try_set` 
  * Sets the value and return the previous value as an Option
  * Will not work if the value is borrowed.
* `replace` and `try_replace` 
  * Sets the value, return the previous value directly. 
  * Will not work on empty cells.
  * Will not work if the value is borrowed.
* `clear` and `try_clear`
  * Clear the value and return the previous value as an Option
  * Will not work if the value is borrowed.
* `get_and_clear` and `try_get_and_clear`
  * Clears the value and return the previous value directly.
  * Will not work if the value is borrowed.
  * Will not work on empty cells.
* `compute` and `try_compute`
  * Calls a Fn with the value to conditionally calculate a new value as a replacement.
  * Will not work if the value is borrowed.
* `compute_if_present` and `try_compute_if_present`
  * Calls a Fn with the value to conditionally calculate a new value as a replacement.
  * Will not work if the value is borrowed.
  * Noop on empty cells.
* `compute_if_absent`
  * Calls a Fn to calculate a new value.
  * Noop on empty cells or cells that have a borrowed value.
* `swap` and `try_swap`
  * Just like `RefCell::swap` 
  * Will swap values between 2 cells.

#### Access:
* `get_and_clone` and `try_get_and_clone` 
  * Returns a clone of the current value in the cell directly.
  * Will not work if the value is borrowed mutably.
  * Will not work on empty cells.
  * Will only be available on Types that implement the Clone trait.
* `if_present_mut`, `if_present`, `try_if_present` and `try_if_present_mut`
  * Call Fn with a reference to the value if the cell is not empty.
  * Noop on empty Cells.
  * mut variant will not work if the value is borrowed
  * normal variant will not work if the value is borrowed mutably.
* `map`, `map_mut`, `try_map`, `try_map_mut`
  * Call Fn with a reference to the value to perform a type conversion.
  * Signature of Fn is `Fn(&T) -> X` or `Fn(&mut T) -> X` 
    * X is the result type of the conversion 
    * T the type in the cell.
  * Entire method `Option<X>` or `Result<Option<X>, RcOCellError>`
  * Will not work on empty cells.
  * mut variant will not work if the value is borrowed.
  * normal variant will not work if the value is borrowed mutably.

### Conversion:
* `T` can convert to `RcOCell<T>` via `into`
* `RcOCell<Vec<T>>` can convert to `Vec<T>` via `try_into`
* `RcOCell<T>` can convert to `Rc<RefCell<Option<T>>>` via `into`
* `Rc<RefCell<Option<T>>>` can convert to `RcOCell<T>` via `into`
* `RcOCell<T>` can convert to `WeakRcOCell<T>` via `into`
  * a dedicated `downgrade` method also exists just like `Rc::downgrade`
* `WeakRcOCell<T>` can convert to `RcOCell<T>` via `try_into`
  * a dedicated `upgrade` method also exists just like `Rc::upgrade`
* `RcOCell<T>` can convert to `Option<T>` via `try_into`
* `RcOCell<T>` can convert to `Result<Option<T>, RcOCellError>` via `into`
* The 'error' types from the normal `RefCell` borrow methods can convert to `RcOCellError` via `into` or the `?` operator.

### Constructors:
* `from_value` and `from`
  * Create a new cell with value, one comes from the `From<T>` trait
* `from_option`
  * Takes an Option as parameter and create a cell with or without value.
* `new`
  * Makes an empty cell

### Misc
* `downgrade` and `upgrade`
  * Conversion between Weak and Normal referenced cell
* `clone`
  * Increases the reference count just like `Rc::clone`. 

## Bigger Example
```rust
pub fn main() {
    let cell : RcOCell<u8> = RcOCell::from_value(1u8); //RcOCell::new() creates a empty cell.

    //Remove the value from the cell
    let old_value : Option<u8> = cell.clear(); //old value would be 1u8.
    let borrow_result : Result<u8, RcOCellError> = cell.try_borrow();
    if borrow_result.is_err() {
        //Would be error because cell is empty.
        //If you want to handle the RcOCellError error, it's an enum. 
        //Can be handled like this:
        match borrow_result.unwrap_err() {
            RcOCellError::NoValue => {println!("No value present")},
            RcOCellError::BorrowError(_) => {}, //Won't happen in this case
            RcOCellError::Dropped => {}, //Won't happen in this case
        }
    }

    //Now cell has value 2. old_value is None.
    let old_value : u8 = cell.set(2u8);
    let borrowed : Ref<u8> = cell.borrow();
    //set would panic, because the value is still borrowed, try_set will fail with RcOCellError::BorrowError
    let try_set_result : Result<u8, RcOCellError> = cell.try_set(4u8);
    drop(borrowed);
    //Now it will work!
    let try_set_result : Result<u8, RcOCellError> = cell.try_set(4u8);
    //2u8 is the old value, 4u8 is now in the cell
    let old_value : u8 = try_set_result.unwrap();
    let current_value : u8 = cell.get_and_clone(); //Only works for all T that implement Clone trait, in this case 4u8
    let current_value : u8 = cell.get_and_clear(); //Cell is empty again after this call. current_value 4u8
    //Another call to get_and_clear would panic as cell is empty.
    let get_result : Result<u8, RcOCellError> = cell.try_get_and_clear();
    //Result is once again RcOCellError::NoValue Error
}
```