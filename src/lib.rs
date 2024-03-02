use core::fmt;
use std::cell::{BorrowError, BorrowMutError, Ref, RefCell, RefMut};
use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;
use std::rc::{Rc, Weak};


pub enum RcOCellComputeResult<T> {
    //Replace the value
    Replace(T),
    //Remove the value (if exists, otherwise noop)
    Remove,
    //Do nothing
    DoNothing
}

pub enum RcOCellBorrowError {
    ///
    /// Normal borrow failed because a mutable borrow already exists somewhere.
    ///
    Normal(BorrowError),
    ///
    /// Mutable borrow failed because a borrow (mutable or not) already exists somewhere
    ///
    Mut(BorrowMutError)
}




pub enum RcOCellError {
    ///
    /// There is no value in the cell
    ///
    NoValue,

    ///
    /// The value is already dropped.
    /// Can only occur when using a Weak Cell.
    ///
    Dropped,
    ///
    /// Failed to borrow the value in the cell
    ///
    BorrowError(RcOCellBorrowError)
}

impl Debug for RcOCellError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        return match self {
            RcOCellError::NoValue => f.write_str("No value present"),
            RcOCellError::BorrowError(e) => Debug::fmt(e, f),
            RcOCellError::Dropped => f.write_str("Cell already dropped"),
        };
    }
}
impl Display for RcOCellError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        return match self {
            RcOCellError::NoValue => f.write_str("No value present"),
            RcOCellError::BorrowError(e) => Display::fmt(e, f),
            RcOCellError::Dropped => f.write_str("Cell already dropped"),
        };
    }
}

unsafe impl Send for RcOCellError {

}

unsafe impl Send for RcOCellBorrowError {

}


impl Debug for RcOCellBorrowError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        return match self {
            RcOCellBorrowError::Normal(e) => Debug::fmt(e, f),
            RcOCellBorrowError::Mut(e) => Debug::fmt(e, f)
        };
    }
}
impl Display for RcOCellBorrowError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        return match self {
            RcOCellBorrowError::Normal(e) => Display::fmt(e, f),
            RcOCellBorrowError::Mut(e) => Display::fmt(e, f)
        };
    }
}

impl From<BorrowError> for RcOCellError {
    fn from(value: BorrowError) -> Self {
        return RcOCellError::BorrowError(RcOCellBorrowError::Normal(value));
    }
}

impl From<BorrowMutError> for RcOCellError {
    fn from(value: BorrowMutError) -> Self {
        return RcOCellError::BorrowError(RcOCellBorrowError::Mut(value));
    }
}

impl <T> From<Rc<RefCell<Option<T>>>> for RcOCell<T>
{
    fn from(value: Rc<RefCell<Option<T>>>) -> Self {
        return RcOCell{rc: value};
    }
}

impl <T> Into<Rc<RefCell<Option<T>>>> for RcOCell<T>
{

    fn into(self) -> Rc<RefCell<Option<T>>> {
        self.rc
    }
}

impl <T> TryInto<Rc<RefCell<Option<T>>>> for WeakRcOCell<T> {
    type Error = RcOCellError;

    fn try_into(self) -> Result<Rc<RefCell<Option<T>>>, Self::Error> {
        Ok(self.try_upgrade()?.rc)
    }
}

impl <T> From<Rc<RefCell<Option<T>>>> for WeakRcOCell<T> {
    fn from(value: Rc<RefCell<Option<T>>>) -> Self {
        return WeakRcOCell{rc: Rc::downgrade(&value)};
    }
}

///
/// This struct represents a mutable reference counted reference to a value that can be present or absent.
/// It has the same borrow checking semantics as RefCell (i.e. Runtime borrow checking)
///
#[derive(Debug)]
pub struct RcOCell<T> where
{
    rc: Rc<RefCell<Option<T>>>
}

#[derive(Debug)]
pub struct WeakRcOCell<T> where
{
    rc: Weak<RefCell<Option<T>>>
}

impl <T> Default for RcOCell<T> where
    T: Default
{
    fn default() -> Self {
        RcOCell::from(T::default())
    }
}

impl <T> Display for RcOCell<T> where
    T: Display
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let x = self.try_borrow();
        if x.is_ok() {
            return Display::fmt(x.unwrap().deref(), f);
        }

        return match x.map(|_| ()).unwrap_err() {
            RcOCellError::NoValue => f.write_str("No value present"),
            RcOCellError::BorrowError(_) => f.write_str("Value currently inaccessible because it is borrowed mutably somewhere"),
            RcOCellError::Dropped => f.write_str("Value already dropped"),
        };
    }
}

impl <T> Clone for RcOCell<T>
{
    fn clone(&self) -> Self {
        return RcOCell { rc: self.rc.clone()};
    }
}

impl <T> From<T> for RcOCell<T> {
    fn from(value: T) -> Self {
        Self::from_value(value)
    }
}

impl <T> Into<Result<T, RcOCellError>> for RcOCell<T> {
    fn into(self) -> Result<T, RcOCellError> {
        self.try_get_and_clear()
    }
}

impl <T> TryInto<Option<T>> for RcOCell<T> {
    type Error = RcOCellError;

    fn try_into(self) -> Result<Option<T>, Self::Error> {
        self.try_clear()
    }
}


impl <T> TryInto<Rc<T>> for RcOCell<T> {
    type Error = RcOCellError;

    fn try_into(self) -> Result<Rc<T>, Self::Error> {
        Ok(Rc::new(self.try_get_and_clear()?))
    }
}

impl <T> TryInto<Vec<T>> for RcOCell<Vec<T>> {
    type Error = RcOCellError;

    fn try_into(self) -> Result<Vec<T>, Self::Error> {
        self.try_get_and_clear()
    }
}

impl <T> RcOCell<T>
{
    ///
    /// Constructs a new empty/cleared RcOCell
    ///
    pub fn new() -> RcOCell<T> {
        return RcOCell {rc: Rc::new(RefCell::new(None))}
    }

    ///
    /// Constructs a new RcOCell from a value.
    ///
    pub fn from_value(value: T) -> RcOCell<T> {
        return RcOCell {rc: Rc::new(RefCell::new(Some(value)))}
    }

    ///
    /// Constructs a new RcOCell from an option either with or without a value depending on the option.
    ///
    pub fn from_option(value: Option<T>) -> RcOCell<T> {
        return RcOCell {rc: Rc::new(RefCell::new(value))}
    }

    ///
    /// Borrows the value mutably.
    /// Panics if the value is already borrowed somewhere (either non mutably or mutably) or there is no value
    ///
    pub fn try_borrow_mut(&self) -> Result<RefMut<T>, RcOCellError> {
        let borrowed = self.rc.as_ref().try_borrow_mut()?;

        if borrowed.is_none() {
            return Err(RcOCellError::NoValue);
        }

        return Ok(RefMut::map(borrowed, |a| a.as_mut().unwrap()));
    }

    ///
    /// Borrows the value.
    /// Fails if the value is already borrowed mutably somewhere or there is no value
    ///
    pub fn try_borrow(&self) -> Result<Ref<T>, RcOCellError> {
        let borrowed = self.rc.as_ref().try_borrow()?;

        if borrowed.is_none() {
            return Err(RcOCellError::NoValue);
        }

        return Ok(Ref::map(borrowed, |a| a.as_ref().unwrap()));
    }

    ///
    /// Borrows the value.
    /// Panics if the value is already borrowed mutably somewhere or there is no value
    ///
    pub fn borrow(&self) -> Ref<T> {
        Ref::map(self.rc.as_ref().borrow(), |a| a.as_ref().unwrap())
    }

    ///
    /// Borrows the value mutably.
    /// Panics if the value is already borrowed somewhere (either non mutably or mutably) or there is no value
    ///
    pub fn borrow_mut(&self) -> RefMut<T> {
        RefMut::map(self.rc.as_ref().borrow_mut(), |a| a.as_mut().unwrap())
    }

    ///
    /// Returns true if the value is set.
    /// Never panics.
    ///
    pub fn is_some(&self) -> bool {
        let borrow = self.rc.try_borrow();
        if borrow.is_err() {
            //Something is borrowed, meaning something exists
            return true;
        }

        return borrow.unwrap().is_some();
    }

    ///
    /// Returns true if the value is not set.
    /// Never panics.
    ///
    pub fn is_none(&self) -> bool {
        let borrow = self.rc.try_borrow();
        if borrow.is_err() {
            //Something is borrowed, meaning something exists
            return false;
        }

        return borrow.unwrap().is_none();
    }

    ///
    /// Runs the Fn with the ref to the value (if present), conditionally creating/updating/removing it.
    /// Panics if the value was borrowed elsewhere.
    ///
    pub fn compute<F>(&self, f: F)
        where F: FnOnce(Option<&mut T>) -> RcOCellComputeResult<T>
    {
        let mut x = self.rc.borrow_mut();
        let result = f(x.as_mut());
        drop(x);
        match result {
            RcOCellComputeResult::Replace(t) => {self.set(t);}
            RcOCellComputeResult::Remove => {self.clear();}
            RcOCellComputeResult::DoNothing => {}
        }
    }

    ///
    /// Runs the Fn with the ref to the value (if present), conditionally creating/updating/removing it.
    /// Panics if the value was borrowed elsewhere.
    ///
    pub fn try_compute<F>(&self, f: F) -> Result<(), RcOCellError>
        where F: FnOnce(Option<&mut T>) -> RcOCellComputeResult<T>
    {
        let mut x = self.rc.try_borrow_mut()?;
        let result = f(x.as_mut());
        drop(x);
        match result {
            RcOCellComputeResult::Replace(t) => {self.set(t);}
            RcOCellComputeResult::Remove => {self.clear();}
            RcOCellComputeResult::DoNothing => {}
        }

        return Ok(());
    }


    ///
    /// Runs the Fn if the value is present to perform a calculation on it, conditionally updating/removing it.
    /// Returns true if the Fn was executed.
    /// False if the value was present.
    /// Panics if the value was borrowed elsewhere.
    ///
    pub fn compute_if_present<F>(&self, f: F) -> bool
        where F: FnOnce(&mut T) -> RcOCellComputeResult<T>
    {
        let mut x = self.rc.borrow_mut();
        if x.is_none() {
            return false;
        }
        let result = f(x.as_mut().unwrap());
        drop(x);
        match result {
            RcOCellComputeResult::Replace(t) => {self.set(t);}
            RcOCellComputeResult::Remove => {self.clear();}
            RcOCellComputeResult::DoNothing => {}
        }

        return true;
    }

    ///
    /// Runs the Fn if the value is present to perform a calculation on it, conditionally updating/removing it.
    /// Returns true if the Fn was executed.
    /// False if the value was present.
    /// Fails if the value was borrowed elsewhere.
    ///
    pub fn try_compute_if_present<F>(&self, f: F) -> Result<bool, RcOCellError>
        where F: FnOnce(&mut T) -> RcOCellComputeResult<T>
    {
        let mut x = self.rc.try_borrow_mut()?;
        if x.is_none() {
            return Ok(false);
        }
        let result = f(x.as_mut().unwrap());
        drop(x);
        match result {
            RcOCellComputeResult::Replace(t) => {self.set(t);}
            RcOCellComputeResult::Remove => {self.clear();}
            RcOCellComputeResult::DoNothing => {}
        }

        return Ok(true);
    }

    ///
    /// Runs the Fn if the value is absent to calculate a new value.
    /// Returns true if the Fn was executed.
    /// False if the value was present or borrowed elsewhere (it also exists in this case).
    /// This function does not panic.
    ///
    pub fn compute_if_absent<F>(&self, f: F) -> bool
        where F: FnOnce() -> Option<T>
    {
        let x = self.rc.try_borrow_mut();
        if x.is_err() {
            return false;
        }
        let x = x.unwrap();
        if x.is_some() {
            return false;
        }

        let result = f();
        drop(x);
        if result.is_some() {
            self.set(result.unwrap());
        }

        return true;
    }

    ///
    /// Runs the Fn if the value is present.
    /// Panics if the value is borrowed mutably elsewhere.
    /// Returns true if the Fn was executed, false if the value was not present.
    ///
    pub fn if_present<F>(&self, f: F) -> bool
        where F: FnOnce(&T) -> RcOCellComputeResult<T> {
        let x = self.rc.borrow();
        if x.is_none() {
            return false;
        }
        f(x.as_ref().unwrap());
        return true;
    }
    ///
    /// Runs the Fn if the value is present.
    /// Panics if the value is borrowed elsewhere.
    /// Returns true if the Fn was executed, false if the value was not present.
    ///
    pub fn if_present_mut<F>(&self, f: F) -> bool
        where F: FnOnce(&mut T) -> RcOCellComputeResult<T> {
        let mut x = self.rc.borrow_mut();
        if x.is_none() {
            return false;
        }
        f(x.as_mut().unwrap());
        return true;
    }

    ///
    /// Runs the Fn if the value is present.
    /// Fails if the value is borrowed mutably elsewhere.
    /// Returns true if the Fn was executed, false if the value was not present.
    ///
    pub fn try_if_present<F>(&self, f: F) -> Result<bool, RcOCellError>
        where F: FnOnce(&T) -> RcOCellComputeResult<T> {
        let x = self.rc.try_borrow()?;
        if x.is_none() {
            return Ok(false);
        }
        f(x.as_ref().unwrap());
        return Ok(true);
    }

    ///
    /// Runs the Fn if the value is present.
    /// Fails if the value is borrowed elsewhere.
    /// Returns true if the Fn was executed, false if the value was not present.
    ///
    pub fn try_if_present_mut<F>(&self, f: F) -> Result<bool, RcOCellError>
        where F: FnOnce(&mut T) -> RcOCellComputeResult<T> {
        let mut x = self.rc.try_borrow_mut()?;
        if x.is_none() {
            return Ok(false);
        }
        f(x.as_mut().unwrap());
        return Ok(true);
    }

    ///
    /// Fetches the value and clears it.
    /// Panics if there is no value or the value is borrowed somewhere.
    ///
    pub fn get_and_clear(&self) -> T {
        let r =  self.rc.replace(None);
        if r.is_none() {
            panic!("RcCell::get_and_clear on a cell without value");
        }

        return r.unwrap();
    }

    ///
    /// Fetches the value and clears it.
    /// Fails if there is no value or if the value is borrowed somewhere.
    ///
    pub fn try_get_and_clear(&self) -> Result<T, RcOCellError> {
        drop(self.rc.try_borrow_mut()?);
        let old = self.rc.replace(None);
        if old.is_none() {
            return Err(RcOCellError::NoValue);
        }

        return Ok(old.unwrap());
    }


    ///
    /// Replaces the value returning the old value.
    /// Panics if there is no value or the value is borrowed somewhere.
    ///
    pub fn replace(&self, value: T) -> T {
        let rep = self.get_and_clear();
        self.rc.replace(Some(value));
        return rep;
    }

    ///
    /// Replaces the value returning the old value.
    /// Fails if there is no value or the value is borrowed somewhere.
    ///
    pub fn try_replace(&self, value: T) -> Result<T, RcOCellError> {
        drop(self.rc.try_borrow_mut()?);
        let rep = self.rc.replace(None);
        if rep.is_none() {
            return Err(RcOCellError::NoValue);
        }
        self.rc.replace(Some(value));
        return Ok(rep.unwrap());
    }

    ///
    /// Sets the value returning the old value (if an old value existed)
    /// Panics if the value is borrowed somewhere.
    ///
    pub fn set(&self, value: T) -> Option<T> {
        return self.rc.replace(Some(value));
    }
    ///
    /// Sets the value returning the old value (if an old value existed)
    /// Fails if the value is borrowed somewhere
    ///
    pub fn try_set(&self, value: T) -> Result<Option<T>, RcOCellError> {
        drop(self.rc.try_borrow_mut()?);
        return Ok(self.set(value));
    }

    ///
    /// Clears the value returning the old value (if an old value existed)
    /// Panics if the value is borrowed somewhere
    ///
    pub fn clear(&self) -> Option<T> {
        return self.rc.replace(None);
    }

    ///
    /// Clears the value returning the old value (if an old value existed)
    /// Fails if the value is borrowed somewhere
    ///
    pub fn try_clear(&self) -> Result<Option<T>, RcOCellError> {
        drop(self.rc.try_borrow_mut()?);
        return Ok(self.clear());
    }

    ///
    /// Calls the Fn with the value (if present) and returns the result as an option.
    /// Panics if the value is already borrowed mutably somewhere.
    /// Returns None if there is no value.
    ///
    pub fn map<F, X>(&self, x: F) -> Option<X> where
        F: FnOnce(&T) -> X,
    {
        let brw = self.rc.as_ref().borrow();
        if brw.is_none() {
            return None
        }

        return Some(x(brw.as_ref().unwrap()));
    }

    ///
    /// Calls the Fn with the value (if present) and returns the result as an option.
    /// Fails if the value is already borrowed mutably somewhere.
    /// Returns None if there is no value.
    ///
    pub fn try_map<F, X>(&self, x: F) -> Result<Option<X>, RcOCellError> where
        F: FnOnce(&T) -> X,
    {
        let brw = self.rc.as_ref().try_borrow()?;
        if brw.is_none() {
            return Ok(None)
        }

        return Ok(Some(x(brw.as_ref().unwrap())));
    }

    ///
    /// Calls the Fn with the mut value (if present) and returns the result as an option.
    /// Panics if the value is already borrowed somewhere.
    /// Returns None if there is no value.
    ///
    pub fn map_mut<F, X>(&self, x: F) -> Option<X> where
        F: FnOnce(&mut T) -> X,
    {
        let mut brw = self.rc.as_ref().borrow_mut();
        if brw.is_none() {
            return None
        }

        return Some(x(brw.as_mut().unwrap()));
    }

    ///
    /// Calls the Fn with the mut value (if present) and returns the result as an option.
    /// Fails if the value is already borrowed somewhere.
    /// Returns None if there is no value.
    ///
    pub fn try_map_mut<F, X>(&self, x: F) -> Result<Option<X>, RcOCellError> where
        F: FnOnce(&mut T) -> X,
    {
        let mut brw = self.rc.as_ref().try_borrow_mut()?;
        if brw.is_none() {
            return Ok(None);
        }

        return Ok(Some(x(brw.as_mut().unwrap())));
    }

    ///
    /// Creates a downgraded version of this cell that only weakly references the cell.
    ///
    pub fn downgrade(&self) -> WeakRcOCell<T> {
        return WeakRcOCell {rc: Rc::downgrade(&self.rc)}
    }

    ///
    /// Swaps the values of both cells.
    /// Panics if either cells value is borrowed
    ///
    pub fn swap(&self, other: &RcOCell<T>) {
        let r = self.rc.as_ref();
        let l = other.rc.as_ref();
        r.swap(l);
    }

    ///
    /// Swaps the values of both cells.
    /// Fails if either cells value is borrowed
    ///
    pub fn try_swap(&self, other: &RcOCell<T>) -> Result<(), RcOCellError>{
        drop(self.try_borrow_mut()?);
        drop(other.try_borrow_mut()?);
        self.swap(other);
        return Ok(());
    }


    ///
    /// Clones the value in the cell
    /// Panics if the cell is empty or the value is currently mutably borrowed
    ///
    pub fn get_and_clone(&self) -> T
        where T: Clone {
        T::clone(&*self.borrow())
    }

    ///
    /// Clones the value in the cell
    /// Fails if the cell is empty or the value is currently mutably borrowed
    ///
    pub fn try_get_and_clone(&self) -> Result<T, RcOCellError>
        where T: Clone {
        Ok(T::clone(&*self.try_borrow()?))
    }
}

impl <T> Clone for WeakRcOCell<T> {
    fn clone(&self) -> Self {
        return WeakRcOCell{rc: self.rc.clone()};

    }
}

impl <T> From<RcOCell<T>> for WeakRcOCell<T> {
    fn from(value: RcOCell<T>) -> Self {
        value.downgrade()
    }
}

impl <T> TryFrom<WeakRcOCell<T>> for RcOCell<T> {
    type Error = RcOCellError;

    fn try_from(value: WeakRcOCell<T>) -> Result<Self, Self::Error> {
        value.try_upgrade()
    }
}


impl <T> WeakRcOCell<T> {
    pub fn upgrade(&self) -> RcOCell<T> {
        let x = self.rc.upgrade();
        if x.is_none() {
            panic!("WeakRcOCell::upgrade called on a dropped cell");
        }

        return RcOCell{rc: x.unwrap()};
    }

    pub fn try_upgrade(&self) -> Result<RcOCell<T>, RcOCellError> {
        let x = self.rc.upgrade();
        if x.is_none() {
            return Err(RcOCellError::Dropped);
        }

        return Ok(RcOCell{rc: x.unwrap()});
    }


    ///
    /// Returns true if the value is set and the cell is not dropped
    /// Never panics.
    ///
    pub fn is_some(&self) -> bool {
        let x = self.rc.upgrade();
        if x.is_none() {
            return false;
        }

        let x = x.unwrap();
        let y = x.try_borrow();
        if y.is_err() {
            return true;
        }

        let y = y.unwrap();
        return y.is_some();
    }

    ///
    /// Returns true if the value is not set or the cell has been dropped
    /// Never panics.
    ///
    pub fn is_none(&self) -> bool {
        let x = self.rc.upgrade();
        if x.is_none() {
            return true;
        }

        let x = x.unwrap();
        let y = x.try_borrow();
        if y.is_err() {
            return false;
        }

        let y = y.unwrap();
        return y.is_none();
    }

    ///
    /// Runs the Fn with the ref to the value (if present), conditionally creating/updating/removing it.
    /// Panics if the value was borrowed elsewhere.
    ///
    pub fn compute<F>(&self, f: F)
        where F: FnOnce(Option<&mut T>) -> RcOCellComputeResult<T>
    {
        self.try_upgrade()
            .expect("WeakRcOCell::compute called on a dropped cell")
            .compute(f)
    }

    ///
    /// Runs the Fn with the ref to the value (if present), conditionally creating/updating/removing it.
    /// Panics if the value was borrowed elsewhere.
    ///
    pub fn try_compute<F>(&self, f: F) -> Result<(), RcOCellError>
        where F: FnOnce(Option<&mut T>) -> RcOCellComputeResult<T>
    {
        self.try_upgrade()?
            .try_compute(f)
    }


    ///
    /// Runs the Fn if the value is present to perform a calculation on it, conditionally updating/removing it.
    /// Returns true if the Fn was executed.
    /// False if the value was present.
    /// Panics if the value was borrowed elsewhere.
    ///
    pub fn compute_if_present<F>(&self, f: F) -> bool
        where F: FnOnce(&mut T) -> RcOCellComputeResult<T>
    {
        self.try_upgrade()
            .expect("WeakRcOCell::compute_if_present called on a dropped cell")
            .compute_if_present(f)
    }

    ///
    /// Runs the Fn if the value is present to perform a calculation on it, conditionally updating/removing it.
    /// Returns true if the Fn was executed.
    /// False if the value was present.
    /// Fails if the value was borrowed elsewhere.
    ///
    pub fn try_compute_if_present<F>(&self, f: F) -> Result<bool, RcOCellError>
        where F: FnOnce(&mut T) -> RcOCellComputeResult<T>
    {
        self.try_upgrade()?
            .try_compute_if_present(f)
    }

    ///
    /// Runs the Fn if the value is absent to calculate a new value.
    /// Returns true if the Fn was executed.
    /// False if the value was present or borrowed elsewhere (it also exists in this case).
    /// Panics if the cell was already dropped.
    ///
    pub fn compute_if_absent<F>(&self, f: F) -> bool
        where F: FnOnce() -> Option<T>
    {
        self.try_upgrade()
            .expect("WeakRcOCell::compute_if_absent called on a dropped cell")
            .compute_if_absent(f)
    }

    ///
    /// Runs the Fn if the value is absent to calculate a new value.
    /// Returns true if the Fn was executed.
    /// False if the value was present or borrowed elsewhere (it also exists in this case).
    /// Fails if the cell was already dropped.
    ///
    pub fn try_compute_if_absent<F>(&self, f: F) -> Result<bool, RcOCellError>
        where F: FnOnce() -> Option<T>
    {
        Ok(self.try_upgrade()?
            .compute_if_absent(f))
    }

    ///
    /// Runs the Fn if the value is present.
    /// Panics if the value is borrowed mutably elsewhere.
    /// Returns true if the Fn was executed, false if the value was not present.
    ///
    pub fn if_present<F>(&self, f: F) -> bool
        where F: FnOnce(&T) -> RcOCellComputeResult<T> {
        self.try_upgrade()
            .expect("WeakRcOCell::if_present called on a dropped cell")
            .if_present(f)
    }
    ///
    /// Runs the Fn if the value is present.
    /// Panics if the value is borrowed elsewhere.
    /// Returns true if the Fn was executed, false if the value was not present.
    ///
    pub fn if_present_mut<F>(&self, f: F) -> bool
        where F: FnOnce(&mut T) -> RcOCellComputeResult<T> {
        self.try_upgrade()
            .expect("WeakRcOCell::if_present_mut called on a dropped cell")
            .if_present_mut(f)
    }

    ///
    /// Runs the Fn if the value is present.
    /// Fails if the value is borrowed mutably elsewhere.
    /// Returns true if the Fn was executed, false if the value was not present.
    ///
    pub fn try_if_present<F>(&self, f: F) -> Result<bool, RcOCellError>
        where F: FnOnce(&T) -> RcOCellComputeResult<T> {
        self.try_upgrade()?.try_if_present(f)
    }

    ///
    /// Runs the Fn if the value is present.
    /// Fails if the value is borrowed elsewhere.
    /// Returns true if the Fn was executed, false if the value was not present.
    ///
    pub fn try_if_present_mut<F>(&self, f: F) -> Result<bool, RcOCellError>
        where F: FnOnce(&mut T) -> RcOCellComputeResult<T> {
        self.try_upgrade()?.try_if_present_mut(f)
    }

    ///
    /// Fetches the value and clears it.
    /// Panics if there is no value or the value is borrowed somewhere.
    ///
    pub fn get_and_clear(&self) -> T {
        self.try_upgrade()
            .expect("WeakRcOCell::get_and_clear called on a dropped cell")
            .get_and_clear()
    }

    ///
    /// Fetches the value and clears it.
    /// Fails if there is no value or if the value is borrowed somewhere.
    ///
    pub fn try_get_and_clear(&self) -> Result<T, RcOCellError> {
        self.try_upgrade()?
            .try_get_and_clear()
    }


    ///
    /// Replaces the value returning the old value.
    /// Panics if there is no value or the value is borrowed somewhere.
    ///
    pub fn replace(&self, value: T) -> T {
        self.try_upgrade()
            .expect("WeakRcOCell::replace called on a dropped cell")
            .replace(value)
    }

    ///
    /// Replaces the value returning the old value.
    /// Fails if there is no value or the value is borrowed somewhere.
    ///
    pub fn try_replace(&self, value: T) -> Result<T, RcOCellError> {
        self.try_upgrade()?
            .try_replace(value)
    }

    ///
    /// Sets the value returning the old value (if an old value existed)
    /// Panics if the value is borrowed somewhere.
    ///
    pub fn set(&self, value: T) -> Option<T> {
        self.try_upgrade()
            .expect("WeakRcOCell::set called on a dropped cell")
            .set(value)
    }
    ///
    /// Sets the value returning the old value (if an old value existed)
    /// Fails if the value is borrowed somewhere
    ///
    pub fn try_set(&self, value: T) -> Result<Option<T>, RcOCellError> {
        self.try_upgrade()?
            .try_set(value)
    }

    ///
    /// Clears the value returning the old value (if an old value existed)
    /// Panics if the value is borrowed somewhere
    ///
    pub fn clear(&self) -> Option<T> {
        self.try_upgrade()
            .expect("WeakRcOCell::clear called on a dropped cell")
            .clear()
    }

    ///
    /// Clears the value returning the old value (if an old value existed)
    /// Fails if the value is borrowed somewhere
    ///
    pub fn try_clear(&self) -> Result<Option<T>, RcOCellError> {
        self.try_upgrade()?
            .try_clear()
    }

    ///
    /// Calls the Fn with the value (if present) and returns the result as an option.
    /// Panics if the value is already borrowed mutably somewhere.
    /// Returns None if there is no value.
    ///
    pub fn map<F, X>(&self, x: F) -> Option<X> where
        F: FnOnce(&T) -> X,
    {
        self.try_upgrade()
            .expect("WeakRcOCell::map called on a dropped cell")
            .map(x)
    }

    ///
    /// Calls the Fn with the value (if present) and returns the result as an option.
    /// Fails if the value is already borrowed mutably somewhere.
    /// Returns None if there is no value.
    ///
    pub fn try_map<F, X>(&self, x: F) -> Result<Option<X>, RcOCellError> where
        F: FnOnce(&T) -> X,
    {
        self.try_upgrade()?
            .try_map(x)
    }

    ///
    /// Calls the Fn with the mut value (if present) and returns the result as an option.
    /// Panics if the value is already borrowed somewhere.
    /// Returns None if there is no value.
    ///
    pub fn map_mut<F, X>(&self, x: F) -> Option<X> where
        F: FnOnce(&mut T) -> X,
    {
        self.try_upgrade()
            .expect("WeakRcOCell::map_mut called on a dropped cell")
            .map_mut(x)
    }

    ///
    /// Calls the Fn with the mut value (if present) and returns the result as an option.
    /// Fails if the value is already borrowed somewhere.
    /// Returns None if there is no value.
    ///
    pub fn try_map_mut<F, X>(&self, x: F) -> Result<Option<X>, RcOCellError> where
        F: FnOnce(&mut T) -> X,
    {
        self.try_upgrade()?
            .try_map_mut(x)
    }

    ///
    /// Clones the value in the cell
    /// Panics if the cell is empty or the value is currently mutably borrowed
    ///
    pub fn get_and_clone(&self) -> T
        where T: Clone {
        self.try_upgrade()
            .expect("WeakRcOCell::map_mut called on a dropped cell")
            .get_and_clone()
    }

    ///
    /// Clones the value in the cell
    /// Fails if the cell is empty or the value is currently mutably borrowed
    ///
    pub fn try_get_and_clone(&self) -> Result<T, RcOCellError>
        where T: Clone {
        self.try_upgrade()?.try_get_and_clone()
    }

}











#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::panic;
    use std::panic::AssertUnwindSafe;
    use std::rc::Rc;
    use crate::*;
    use crate::RcOCellComputeResult::Replace;

    #[test]
    fn test_set_and_reset_new() {
        let x = RcOCell::new();
        assert_eq!(x.is_none(), true);
        assert_eq!(x.is_some(), false);
        assert_eq!(x.set("Baum".to_string()).is_none(), true);
        assert_eq!(x.is_none(), false);
        assert_eq!(x.is_some(), true);
        let str = x.get_and_clear();
        assert_eq!(str.as_str(), "Baum");
        assert_eq!(x.is_none(), true);
        assert_eq!(x.is_some(), false);
    }

    #[test]
    fn test_set_and_reset_from() {
        let x = RcOCell::from_value("Baum".to_string());
        assert_eq!(x.is_none(), false);
        assert_eq!(x.is_some(), true);
        let str = x.get_and_clear();
        assert_eq!(str.as_str(), "Baum");
        assert_eq!(x.is_none(), true);
        assert_eq!(x.is_some(), false);
    }

    #[test]
    fn test_display() {
        let x = RcOCell::from_value("Baum".to_string());
        assert_eq!(format!("{}", x).as_str(), "Baum");
        let brw = x.borrow();
        assert_eq!(format!("{}", x).as_str(), "Baum");
        drop(brw);
        let brw = x.borrow_mut();
        assert_eq!(format!("{}", x).as_str(), "Value currently inaccessible because it is borrowed mutably somewhere");
        drop(brw);
        x.clear();
        assert_eq!(format!("{}", x).as_str(), "No value present");
    }

    #[test]
    fn test_map() {
        let x = RcOCell::from_value("Baum".to_string());
        let y = x.map(|e| {
            assert_eq!(e.as_str(), "Baum");
            return 12345u64;
        })

            .unwrap();
        assert_eq!(y, 12345u64);
        x.clear();
    }

    #[test]
    fn test_conv_result() {
        let z = "Baum".to_string();
        let x: RcOCell<String> = z.clone().into();
        let y: Result<String, RcOCellError> = x.into();
        assert_eq!(y.unwrap().as_str(), "Baum");
        let x: RcOCell<String> = z.into();
        let x2 = x.clone();
        let g = x2.borrow_mut();
        let y: Result<String, RcOCellError> = x.clone().into();
        drop(g);
        assert_eq!(y.is_err(), true);
        match y.unwrap_err() {
            RcOCellError::BorrowError(RcOCellBorrowError::Mut(_)) => {}
            _ => panic!("unexpected"),
        }
        let y: Result<String, RcOCellError> = x.into();
        assert_eq!(y.is_err(), false);
        assert_eq!(y.unwrap().as_str(), "Baum");
    }

    #[test]
    fn test_conv_option() {
        let z = "Baum".to_string();

        let x: RcOCell<String> = z.into();
        let y: Option<String> = x.try_into().unwrap();
        assert_eq!(y.unwrap().as_str(), "Baum");
    }

    #[test]
    fn test_conv_vec() {
        let z = vec!["Baum".to_string(), "Nase".to_string()];
        let base = z.clone();

        let x: RcOCell<Vec<String>> = z.into();
        let y: Vec<String> = x.try_into().unwrap();
        assert_eq!(y, base);
    }

    #[test]
    fn test_replace_panic() {
        let x = RcOCell::from_value("Baum".to_string());
        let y = x.borrow();
        let r = panic::catch_unwind(AssertUnwindSafe(|| {
            x.replace("Nase".to_string());
        }));
        assert_eq!(r.is_err(), true);
        assert_eq!(y.as_str(), "Baum");
        drop(y);
        assert_eq!(format!("{}", x), "Baum".to_string());
        assert_eq!(x.replace("Nase".to_string()), "Baum".to_string());
    }

    #[test]
    fn test_get_empty_panic() {
        let x = RcOCell::from_value("Baum".to_string());
        x.clear();
        let r = panic::catch_unwind(AssertUnwindSafe(|| {
            x.get_and_clear();
        }));
        assert_eq!(r.is_err(), true);
    }


    #[test]
    fn test_set_panic() {
        let x = RcOCell::from_value("Baum".to_string());
        let y = x.borrow();
        let r = panic::catch_unwind(AssertUnwindSafe(|| {
            x.set("Nase".to_string());
        }));
        assert_eq!(r.is_err(), true);
        assert_eq!(y.as_str(), "Baum");
        drop(y);
        assert_eq!(format!("{}", x), "Baum".to_string());
        assert_eq!(x.set("Nase".to_string()).unwrap(), "Baum".to_string());
        assert_eq!(format!("{}", x), "Nase".to_string());
    }

    #[test]
    fn test_clear_panic() {
        let x = RcOCell::from_value("Baum".to_string());
        let y = x.borrow();
        let r = panic::catch_unwind(AssertUnwindSafe(|| {
            x.clear();
        }));
        assert_eq!(r.is_err(), true);
        assert_eq!(y.as_str(), "Baum");
        drop(y);
        assert_eq!(format!("{}", x), "Baum".to_string());
        x.clear();
        assert_eq!(format!("{}", x), "No value present".to_string());
    }


    #[test]
    fn test_borrow_panic() {
        let x = RcOCell::from_value("Baum".to_string());
        let y = x.borrow_mut();
        let r = panic::catch_unwind(AssertUnwindSafe(|| {
            x.borrow_mut();
        }));
        assert_eq!(r.is_err(), true);
        drop(y);
        let y = x.borrow_mut();
        drop(y);

        let r = panic::catch_unwind(AssertUnwindSafe(|| {
            let _m = x.borrow_mut();
            panic!("Oh no");
        }));
        assert_eq!(r.is_err(), true);
        let y = x.borrow_mut();
        drop(y);

        let y = x.borrow();
        let r = panic::catch_unwind(AssertUnwindSafe(|| {
            let _m = x.borrow_mut();
        }));
        assert_eq!(r.is_err(), true);
        drop(y);
        let y = x.borrow_mut();
        drop(y);

        let y = x.borrow_mut();
        let r = panic::catch_unwind(AssertUnwindSafe(|| {
            let _m = x.borrow();
        }));
        assert_eq!(r.is_err(), true);
        drop(y);
        let y = x.borrow();
        drop(y);
    }

    #[test]
    fn test_downgrade() {
        let x = RcOCell::from_value("Baum".to_string());
        let down = x.downgrade();
        assert_eq!(down.is_some(), true);
        assert_eq!(down.try_upgrade().is_ok(), true);
        drop(x);
        assert_eq!(down.is_none(), true);
        assert_eq!(down.try_upgrade().is_err(), true);
    }

    #[test]
    fn test_downgrade_set() {
        let x = RcOCell::from_value("Baum".to_string());
        let down = x.downgrade();
        down.set("Nudel".to_string());
        assert_eq!(x.borrow().as_str(), "Nudel");
        drop(x);
        assert_eq!(down.is_none(), true);
        assert_eq!(down.try_upgrade().is_err(), true);
        let r = panic::catch_unwind(AssertUnwindSafe(|| {
            let _m = down.set("Nudel".to_string());
        }));
        assert_eq!(r.is_err(), true);
    }

    #[test]
    fn test_downgrade_get() {
        let x = RcOCell::from_value("Baum".to_string());
        let down = x.downgrade();
        assert_eq!(down.get_and_clear().as_str(), "Baum");
        assert_eq!(x.is_none(), true);
        x.set("Raum".to_string());
        drop(x);
        assert_eq!(down.is_none(), true);
        assert_eq!(down.try_upgrade().is_err(), true);
        let r = panic::catch_unwind(AssertUnwindSafe(|| {
            let _m = down.get_and_clear();
        }));
        assert_eq!(r.is_err(), true);
    }

    #[test]
    fn test_conv_unwrap() {
        let x = RcOCell::from_value("Baum".to_string());
        let y: Rc<RefCell<Option<String>>> = x.into();
        let br = y.borrow();
        let z = br.as_ref().unwrap();
        assert_eq!(z.as_str(), "Baum");
        drop(br);
        let x: RcOCell<String> = y.into();
        let d = x.downgrade();
        let y: Rc<RefCell<Option<String>>> = d.try_into().expect("Error");
        let br = y.borrow();
        let z = br.as_ref().unwrap();
        assert_eq!(z.as_str(), "Baum");
    }

    #[test]
    fn test_get_clone() {
        let x = RcOCell::from_value("Baum".to_string());
        let y = x.get_and_clone();
        assert_eq!(x.get_and_clear().as_str(), "Baum");
        assert_eq!(y.as_str(), "Baum");
    }

    #[test]
    fn test_compute() {
        let x = RcOCell::from_value(1u32);
        x.compute(|n| Replace(*n.unwrap() * 2));
        assert_eq!(x.get_and_clone(), 2u32);
        assert_eq!(x.compute_if_present(|n| Replace(*n*2)), true);
        assert_eq!(x.get_and_clone(), 4u32);
        assert_eq!(x.compute_if_absent(|| Some(1)), false);
        assert_eq!(x.get_and_clone(), 4u32);
        x.clear();
        assert_eq!(x.compute_if_absent(|| Some(1)), true);
        assert_eq!(x.get_and_clone(), 1u32);
        let _b = x.borrow_mut();
    }

    #[test]
    fn test_clone() {
        let x = RcOCell::from_value(1u32);
        let w = x.downgrade();
        let y = x.clone();
        drop(x);
        assert_eq!(w.get_and_clone(), 1u32);
        assert_eq!(y.get_and_clone(), 1u32);
    }
}
