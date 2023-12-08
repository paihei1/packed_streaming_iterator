//! Some usefull additions to the [`streaming_iterator`] crate.
//!
//! Includes versions of [`StreamingIterator`] adapters that derive [`Clone`], 
//! streaming versions of some more common iterator adapters like 
//! zip and enumerate, and some more streaming iterator adapters.
//!
//! see [`Streaming`] trait for all documentation.

#![warn(missing_docs)]
extern crate streaming_iterator;
use streaming_iterator::{StreamingIterator,StreamingIteratorMut};
use std::ops::{AddAssign,SubAssign};
use std::cmp::{Ord,Ordering};

/// Supertrait implemented for any [`StreamingIterator`], used to provide additional methods.
pub trait Streaming: StreamingIterator+Sized {
    /// Creates a StreamingIterator that gives the current iteration count and Item.
    ///
    /// The Iterator returned yields type `&Enumerate<Self>`, on which calling the [`Enumerate::unwrap`] method returns the tuple `(n,&item)` where `Self::get()` would have yielded `&item`s.
    ///
    /// if `Self: StreamingIteratorMut`, `self.enumerate` is also [`StreamingIteratorMut`],
    /// and to retrieve tuples `(usize,&mut Item)`, `self.enumerate().next_mut().unwrap().unwrap_mut()` can be called.
    ///
    /// # Examples
    /// ```
    /// use streaming_iterator::StreamingIterator;
    /// use packed_streaming_iterator::Streaming;
    /// 
    /// let mut planets = streaming_iterator::convert(["Mercury".to_owned(),"Venus".to_owned(),
    ///                                                "Earth".to_owned(),"Mars".to_owned()])
    ///     .enumerate();
    ///
    /// assert_eq!(planets.next()
    ///     .unwrap() // next() returns Option<_>
    ///     .unwrap() // call unwrap() to retrieve tuple
    /// , (0,&"Mercury".to_owned()));
    /// assert_eq!(planets.next().unwrap().unwrap(),(1,&"Venus".to_owned()));
    ///
    /// ```
    fn enumerate(self) -> Enumerate<Self> {
	Enumerate {count: 0, it: self}
    }
    /// 'Zips up' two streaming iterators into a single streaming iterator of pairs.
    ///
    /// The new iterator returns [`Zip`] objects, on which `unwrap()` can be called to retrieve a tuple `(&I::Item,&J::Item)`. It ends if either of the input iterators ends.
    ///
    /// If both iterators are [`StreamingIteratorMut`], `self.zip(other).next_mut().unwrap().unwrap_mut()` yields `(&mut I::Item,&mut J::Item)`.
    ///
    /// # Examples
    /// ```
    /// use streaming_iterator::StreamingIterator;
    /// use packed_streaming_iterator::Streaming;
    /// 
    /// let mut words = streaming_iterator::convert(["a","bcd","ef"]);
    /// let mut numbers = streaming_iterator::convert([vec![1,2],vec![3,4,5],vec![6]]);
    ///
    /// let mut iter = words.zip(numbers);
    ///
    /// assert_eq!(iter.next()
    ///     .unwrap() // next() returns Option<_>
    ///     .unwrap() // retrieve tuple
    /// ,(&"a",&vec![1,2]));
    /// assert_eq!(iter.next().unwrap().unwrap(),(&"bcd",&vec!(3,4,5)));
    /// 
    /// ```
    fn zip<J: StreamingIterator>(self, j: J) -> Zip<Self,J> {
	Zip {i: self, j}
    }
    /// Maps a streaming iterator over one type to one over another by applying a closure.
    ///
    /// The Reason the [`Iterator::map`] method has multiple analogues for streaming iterators
    /// ([`StreamingIterator::map`],[`StreamingIterator::map_ref`],...), is that with streaming iterators
    /// it is common for the mapped-to type to reference back to the original item.
    /// At the same time we cannot express the corresponding lifetime constraints:
    /// ```compile_fail
    /// # use streaming_iterator::StreamingIterator;
    /// # struct Hidden<F,I: StreamingIterator,T> where
    /// F: for<'a> FnMut(&'a I::Item) -> T where T: 'a
    /// # {i:I,f:F}
    /// ```
    /// therefore [`StreamingIterator::map`] is:
    /// ```
    /// # use streaming_iterator::StreamingIterator;
    /// # struct Hidden<F,I: StreamingIterator,T> where
    /// F: for<'a> FnMut(&'a I::Item) -> T
    /// # {i:I,f:F}
    /// ```
    /// and [`StreamingIterator::map_ref`] is:
    /// ```
    /// # use streaming_iterator::StreamingIterator;
    /// # struct Hidden<F,I: StreamingIterator,T> where
    /// F: for<'a> Fn(&'a I::Item) -> &'a T
    /// # {i:I,f:F}
    /// ```
    /// [`Streaming::map_ref_and`] and [`Streaming::map_mut_and`] bridge the gap by allowing the [`RefAnd`](or [`MutRefAnd`]) type, consisting of a reference and some other value.
    /// The only remaining limitation is only being able to have a single reference back, but this can be circumvented by referencing the entire thing and later splitting it up.
    ///
    /// # Examples
    /// ```
    /// use streaming_iterator::StreamingIterator;
    /// use packed_streaming_iterator::{Streaming,RefAnd};
    /// 
    /// let names = streaming_iterator::convert(["Jimmy","Carl","Jeff"]);
    /// let mut iter = names.map_ref_and(|name| RefAnd::new(name, name.len()));
    ///
    /// assert_eq!(iter.next().unwrap() // next() returns Option<RefAnd<'_,_,_>>
    ///     .unwrap() // retrieve the tuple
    /// , (&"Jimmy", &5));
    /// assert_eq!(iter.next().unwrap().unwrap(),(&"Carl",&4));
    ///
    /// ```
    ///
    /// [`MapRefAnd`] and [`MapMutRefAnd`] also implement [`Clone`] if `Self` and `F` do, correctly pointing the references to the cloned iterator.
    fn map_ref_and<F,R,T>(self, f: F) -> MapRefAnd<Self,F,R,T>
    where F: for<'a> FnMut(&'a Self::Item) -> RefAnd<'a,R,T> {
	MapRefAnd {it: self, f, inner: None}
    }
    /// [`Streaming::map_ref_and`] with a mutable pointer back for [`StreamingIteratorMut`]s.
    ///
    /// see [`Streaming::map_ref_and`].
    ///
    /// If we map to another streaming iterator (if you `impl StreamingIterator for MutRefAnd<'_,_,(your type)>`), [`StreamingIteratorMut::flatten`] works correctly.
    fn map_mut_and<F,R,T>(self, f: F) -> MapMutRefAnd<Self,F,R,T>
    where F: for<'a> FnMut(&'a mut Self::Item) -> MutRefAnd<'a,R,T> {
	MapMutRefAnd {it: self, f, inner: None}
    }
    /// Streaming iterator over all pairs of one Item from `self` and one from a second iterator generated by a function.
    ///
    /// <div class="warning">This adapter does not consume `self`, but holds a mutable reference to it instead. Make sure self stays in scope for the existance of the adapter.</div>
    ///
    /// The closure gets called once for each item from `self`.
    /// 
    /// the yielded Items are of type `MutRefAnd<'_,Self,Combinations<(other iterator),(generating function)>>` and still have to be `unwrap_full()`'ed to get `(&Self::Item, &J::Item)`.
    ///
    /// # Examples
    /// ```
    /// use streaming_iterator::StreamingIterator;
    /// use packed_streaming_iterator::{Streaming,RefAnd};
    /// 
    /// let mut a = streaming_iterator::convert([1,2,3]);
    /// let mut iter = a.combinations_with(|| {
    ///     streaming_iterator::convert(['a','b','c'])
    /// });
    ///
    /// let result: Vec<(usize,char)> = iter.map(|combination| {
    ///         let (ref_a,ref_b) = combination.unwrap_full();
    ///         (*ref_a,*ref_b)
    ///     }).cloned().collect();
    ///
    /// assert_eq!(result, vec![(1,'a'),(1,'b'),(1,'c'),
    ///                         (2,'a'),(2,'b'),(2,'c'),
    ///                         (3,'a'),(3,'b'),(3,'c')]);
    /// 
    /// ```
    fn combinations_with<'a,J,F>(&'a mut self, mut f: F)
				 -> MutRefAnd<'a,Self,Combinations<J,F>>
    where J: StreamingIterator, F: FnMut() -> J {
	self.advance();
	MutRefAnd {r: self, other: Combinations {j: f(),f}}
    }
    /// Streaming iterator adapter to find selections of items from one cloneable streaming iterator.
    ///
    /// A function f is used to calculate a `T` for the `Self::Item`s, where `T` implements [`std::cmp::Ord`]. `self` **has to be ordered** by this total ordering, from smallest to largest. `T` also has to implement [`std::ops::AddAssign<T>`] and [`std::ops::SubAssign<T>`], the inverse operation.
    ///
    /// `inner_combinations` then iterates over **the ordered sets of** `Self::Item`**s with repeats, for which the** `T` **values sum to** `target`.
    ///
    /// Usually, the `stack` argument will be `Vec::new()` of type `Vec<Self>`. In the Example `T` is `usize`.
    ///
    /// # Examples
    /// ```
    /// use streaming_iterator::StreamingIterator;
    /// use packed_streaming_iterator::{Streaming,RefAnd};
    /// 
    /// let mut a = streaming_iterator::convert([vec![1],vec![0],vec![3,2],vec![4,5,6]]);
    /// let mut iter = a.inner_combinations(4,|s| s.len(), Vec::new());
    ///
    /// let result: Vec<_> = iter.map(|combination| { // combination: &Vec<&(type of a)>
    ///         combination.into_iter().map(|i| i.get()
    ///             // i is of type type_of(a), use get() to get type_of(a)::Item 
    ///             .expect("these are guaranteed to be Some() if iter.get() returned Some().").clone())
    ///             .collect::<Vec<_>>()
    ///     }).cloned().collect();
    ///
    /// assert_eq!(result, vec![vec![vec![1],vec![1],vec![1],vec![1]],
    ///                         vec![vec![1],vec![1],vec![1],vec![0]],
    ///                         vec![vec![1],vec![1],vec![0],vec![0]],
    ///                         vec![vec![1],vec![1],vec![3,2]],
    ///                         vec![vec![1],vec![0],vec![0],vec![0]],
    ///                         vec![vec![1],vec![0],vec![3,2]],
    ///                         vec![vec![1],vec![4,5,6]],
    ///                         vec![vec![0],vec![0],vec![0],vec![0]],
    ///                         vec![vec![0],vec![0],vec![3,2]],
    ///                         vec![vec![0],vec![4,5,6]],
    ///                         vec![vec![3,2],vec![3,2]]]);
    /// // all ordered  combinations with a combined length of 4.
    /// ```
    ///
    /// If you implement [`Stack`] for your own type such that [`Stack::try_push`] might fail, e.g. when the stack is full, `inner_combinations()` will correctly iterate over all combinations which consist of a number of elements that does fit.
    fn inner_combinations<T,F,S>(mut self, mut target: T, f: F, mut stack: S)
				 -> InnerCombinations<T,F,S>
    where Self: Clone, F: Fn(&Self::Item) -> T, S: Stack<Item=Self>,
	  T: AddAssign<T> + SubAssign<T> + Ord + Clone {
	if let Some(first) = self.next() {
	    target -= f(first);
	} else {
	    return InnerCombinations {f,target,stack}
	}
	stack.try_push(self);
	InnerCombinations {f,target,stack}
    }
    /// A copy of [`StreamingIterator::filter`] where the output derives [`Clone`]
    fn cfilter<F: FnMut(&Self::Item)->bool>(self,f:F) -> CFilter<Self,F> {
	CFilter{it: self, f}
    }
    /// A copy of [`StreamingIterator::map`] where the output derives [`Clone`]
    fn cmap<F: FnMut(&Self::Item)->B,B>(self,f:F) -> CMap<Self,F,B> {
	CMap {it: self, f, b: None}
    }
    /// A copy of [`StreamingIterator::flat_map`] where the output derives [`Clone`]
    fn cflat_map<F,J>(self, f:F) -> CFlatMap<Self,F,J>
    where J: StreamingIterator, F: FnMut(&Self::Item) -> J {
	CFlatMap {it: self, f, j: None}
    }
}

/// Enumerating adapter for streaming iterators
///
/// see [`Streaming::enumerate`]
#[derive(Debug,Clone)]
pub struct Enumerate<I: StreamingIterator> {
    count: usize,
    it: I,
}
impl<I: StreamingIterator> StreamingIterator for Enumerate<I> {
    type Item = Self;
    #[inline]fn advance(&mut self) {
	self.it.advance();
	self.count += 1;
    }
    #[inline]fn get(&self) -> Option<&Self::Item> {
	if self.it.is_done() {None} else {Some(self)}
    }
    #[inline]fn is_done(&self) -> bool {self.it.is_done()}
}
impl<I: StreamingIteratorMut> StreamingIteratorMut for Enumerate<I> {
    #[inline]fn get_mut(&mut self) -> Option<&mut Self::Item> {
	if self.it.is_done() {None} else {Some(self)}
    }
}
impl<I: StreamingIterator> Enumerate<I> {
    /// Enumerate returns itself from the `get()` and `next()` methods,
    /// to retrieve a tuple ```(n,&<inner item>)```, `unwrap` has to be called
    #[inline]pub fn unwrap<'a>(&'a self) -> (usize, &'a I::Item) {
	(self.count-1, self.it.get().unwrap())
    }
}
impl<I: StreamingIteratorMut> Enumerate<I> {
    /// Enumerate returns itself from the `get_mut()` and `next_mut()` methods,
    /// to retrieve a tuple `(n,&mut inner item)`, `unwrap_mut` has to be called
    #[inline]pub fn unwrap_mut<'a>(&'a mut self) -> (usize, &'a mut I::Item) {
	(self.count-1, self.it.get_mut().unwrap())
    }
}

/// Streaming Iterator adapter yielding pairs of items from two iterators
///
/// see [`Streaming::zip`]
#[derive(Debug,Clone)]
pub struct Zip<I,J> {i: I, j: J}
impl<I,J> StreamingIterator for Zip<I,J>
where I: StreamingIterator, J: StreamingIterator {
    type Item = Self;
    #[inline]fn advance(&mut self) {self.i.advance();self.j.advance();}
    #[inline]fn get(&self) -> Option<&Self::Item> {
	(!(self.is_done())).then_some(self)
    }
    #[inline]fn is_done(&self) -> bool {self.i.is_done() || self.j.is_done()}
}
impl<I,J> StreamingIteratorMut for Zip<I,J>
where I: StreamingIteratorMut, J: StreamingIteratorMut {
    #[inline]fn get_mut(&mut self) -> Option<&mut Self::Item> {
	(!(self.is_done())).then_some(self)
    }
}
impl<I,J> Zip<I,J>
where I: StreamingIterator, J: StreamingIterator {
    /// Zip returns itself from the `get()` and `next()` methods,
    /// to retrieve a tuple ```(&<Item>,&<Item>)```, `unwrap` has to be called
    #[inline]pub fn unwrap(&self) -> (&I::Item, &J::Item) {
	(self.i.get().unwrap(),self.j.get().unwrap())
    }
}
impl<I,J> Zip<I,J>
where I: StreamingIteratorMut, J: StreamingIteratorMut {
    /// Zip returns itself from the `get_mut()` and `next_mut()` methods,
    /// to retrieve a tuple ```(&mut <Item>,&mut <Item>)```, `unwrap_mut` has to be called
    #[inline]pub fn unwrap_mut(&mut self) -> (&mut I::Item, &mut J::Item) {
	(self.i.get_mut().unwrap(),self.j.get_mut().unwrap())
    }
}
/// A struct encapsulating a reference and another object.
///
/// see [`Streaming::map_ref_and`]
#[derive(Debug)]
pub struct RefAnd<'a,R,T> {r: &'a R, other: T}
impl<'a,R:'static,T> RefAnd<'a,R,T> {
    #[inline]unsafe fn to_static(self) -> RefAnd<'static,R,T> {
	let RefAnd{r,other} = self;
	RefAnd{ r: &*(r as *const R), other}
    }
    /// Create a [`RefAnd`]
    #[inline]pub fn new(r: &'a R, other: T) -> Self {Self {r,other}}
    /// The methods `get()` and `next()` on `MapRefAnd` return `RefAnd`,
    /// to retrieve the contents, call `unwrap`.
    #[inline]pub fn unwrap<'b>(&'b self) -> (&'b R, &'b T) {
	(self.r,&self.other)
    }
}
/// Iterator adapter to map to a combination of a reference into the original Iterator and also a new object.
///
/// see `Streaming::map_ref_and`
#[derive(Debug)]
pub struct MapRefAnd<I,F,R:'static,T> {
    it: I, f: F, inner: Option<RefAnd<'static,R,T>>
}
impl<I,F,R: 'static,T> StreamingIterator for MapRefAnd<I,F,R,T>
where I: StreamingIterator, F: for<'a> FnMut(&'a I::Item) -> RefAnd<'a,R,T> {
    type Item = RefAnd<'static,R,T>;
    #[inline]fn advance(&mut self) {
	self.inner = self.it.next().map(|next| unsafe {
	    ((self.f)(next)).to_static()
	});
    }
    #[inline]fn get(&self) -> Option<&Self::Item> {
	self.inner.as_ref()
    }
}
impl<I,F,R,T> Clone for MapRefAnd<I,F,R,T>
where I: StreamingIterator+Clone, T: Clone,
      F: Clone+for<'a> Fn(&'a I::Item) -> RefAnd<'a,R,T>, {
    fn clone(&self) -> Self {
	let it = self.it.clone();
	let f = self.f.clone();
	let inner = if let Some(ref inner) = self.inner {
	    Some(RefAnd{r: unsafe {f(it.get().unwrap()).to_static()}.r,
			other: inner.other.clone()})
	} else {None};
	Self {it,f,inner}
    }
}

/// A struct encapsulating a mutable reference and another object.
///
/// see [`Streaming::map_mut_and`]
#[derive(Debug)]
pub struct MutRefAnd<'a,R,T> {r: &'a mut R, other: T}
impl<'a,R:'static,T> MutRefAnd<'a,R,T> {
    #[inline]unsafe fn to_static(self) -> MutRefAnd<'static,R,T> {
	let MutRefAnd{r,other} = self;
	MutRefAnd{ r: &mut *(r as *mut R), other}
    }
    /// Create a [`MutRefAnd`]
    #[inline]pub fn new(r: &'a mut R, other: T) -> Self {Self {r,other}}
    /// The methods `get()` and `next()` on [`MapMutRefAnd`] return `&MutRefAnd`,
    /// to retrieve the contents, call `unwrap`.
    #[inline]pub fn unwrap<'b>(&'b self) -> (&'b R, &'b T) {
	(self.r,&self.other)
    }
    /// The methods `get_mut()` and `next_mut()` on [`MapMutRefAnd`] return `&mut MutRefAnd`,
    /// to retrieve the contents, call `unwrap_mut`.
    #[inline]pub fn unwrap_mut<'b>(&'b mut self) -> (&'b mut R, &'b mut T) {
	(self.r,&mut self.other)
    }
}
/// Iterator adapter to map to a combination of a mutable reference into the original Iterator and also a new object.
///
/// see `Streaming::map_mut_and`
#[derive(Debug)]
pub struct MapMutRefAnd<I,F,R:'static,T> {
    it: I, f: F, inner: Option<MutRefAnd<'static,R,T>>
}
impl<I,F,R: 'static,T> StreamingIterator for MapMutRefAnd<I,F,R,T>
where I: StreamingIteratorMut,
      F: for<'a> FnMut(&'a mut I::Item) -> MutRefAnd<'a,R,T> {
    type Item = MutRefAnd<'static,R,T>;
    #[inline]fn advance(&mut self) {
	self.inner = self.it.next_mut().map(|next| unsafe {
	    ((self.f)(next)).to_static()
	});
    }
    #[inline]fn get(&self) -> Option<&Self::Item> {
	self.inner.as_ref()
    }
}
impl<I,F,R: 'static,T> StreamingIteratorMut for MapMutRefAnd<I,F,R,T>
where I: StreamingIteratorMut,
      F: for<'a> FnMut(&'a mut I::Item) -> MutRefAnd<'a,R,T> {
    #[inline]fn get_mut(&mut self) -> Option<&mut Self::Item> {
	self.inner.as_mut()
    }
}
impl<I,F,R,T> Clone for MapMutRefAnd<I,F,R,T>
where I: StreamingIteratorMut+Clone, T: Clone,
      F: Clone+for<'a> Fn(&'a mut I::Item) -> MutRefAnd<'a,R,T>, {
    fn clone(&self) -> Self {
	let mut it = self.it.clone();
	let f = self.f.clone();
	let inner = if let Some(ref inner) = self.inner {
	    Some(MutRefAnd{r: unsafe {f(it.get_mut().unwrap()).to_static()}.r,
			   other: inner.other.clone()})
	} else {None};
	Self {it,f,inner}
    }
}
/// Part of an iterator adapter to iterate over all possible pairs from two iterators.
///
/// see [`Streaming::combinations_with`]
#[derive(Debug,Clone)]
pub struct Combinations<J,F> {j: J, f: F}
impl<'a,I,J,F> StreamingIterator for MutRefAnd<'a,I,Combinations<J,F>>
where I: StreamingIterator, J: StreamingIterator, F: FnMut() -> J {
    type Item = Self;
    #[inline]fn advance(&mut self) {
	self.other.j.advance();
	if !(self.other.j.is_done()) {return}
	self.r.advance();
	self.other.j = (self.other.f)();
	self.other.j.advance();
    }
    #[inline]fn get(&self) -> Option<&Self::Item> {
	(!(self.is_done())).then_some(self)
    }
    #[inline]fn is_done(&self) -> bool {
	self.r.is_done() || self.other.j.is_done()
    }
}
impl<'a,I,J,F> MutRefAnd<'a,I,Combinations<J,F>>
where I: StreamingIterator, J: StreamingIterator {
    /// The methods `get()` and `next()` on the iterator returned from [`Streaming::combinations_with`]
    /// return this, to retrieve actual value pairs call `unwrap_full`.
    #[inline]pub fn unwrap_full<'b>(&'b self) -> (&'b I::Item, &'b J::Item) {
	(self.r.get().unwrap(),self.other.j.get().unwrap())
    }
}
/// A version of [`streaming_iterator::Filter`] that derives [`Clone`].
///
/// see [`Streaming::cfilter`]
#[derive(Debug,Clone)]
pub struct CFilter<I,F> {
    it: I, f: F
}
impl<I,F> StreamingIterator for CFilter<I,F>
where I:StreamingIterator,F: FnMut(&I::Item) -> bool {
    type Item = I::Item;
    #[inline]fn advance(&mut self) {
	while let Some(i) = self.it.next() {if (self.f)(i) {break}}
    }
    #[inline]fn is_done(&self) -> bool {self.it.is_done()}
    #[inline]fn get(&self) -> Option<&I::Item> {self.it.get()}
    #[inline]fn size_hint(&self)->(usize, Option<usize>){
	(0, self.it.size_hint().1)
    }
}

/// A version of [`streaming_iterator::Map`] that derives [`Clone`].
///
/// see [`Streaming::cmap`]
#[derive(Debug,Clone)]
pub struct CMap<I,F,B> {it: I, f: F, b: Option<B>}
impl<I,F,B> StreamingIterator for CMap<I,F,B>
where I: StreamingIterator, F: FnMut(&I::Item) -> B {
    type Item = B;
    #[inline]fn advance(&mut self) {self.b=self.it.next().map(&mut self.f);}
    #[inline]fn get(&self) -> Option<&B> {self.b.as_ref()}
    #[inline]fn size_hint(&self) -> (usize, Option<usize>) {self.it.size_hint()}
}
impl<I,F,B> StreamingIteratorMut for CMap<I,F,B>
where I: StreamingIterator, F: FnMut(&I::Item) -> B {
    #[inline]fn get_mut(&mut self) -> Option<&mut B> {self.b.as_mut()}
}

/// A version of [`streaming_iterator::FlatMap`] that derives [`Clone`].
///
/// see [`Streaming::cflat_map`]
#[derive(Debug,Clone)]
pub struct CFlatMap<I,F,J> {it: I, f: F, j: Option<J>}
impl<I,F,J> StreamingIterator for CFlatMap<I,F,J>
where I: StreamingIterator, F: FnMut(&I::Item) -> J, J: StreamingIterator {
    type Item = J::Item;
    #[inline]fn advance(&mut self) { loop {
	if let Some(ref mut iter) = self.j {
            iter.advance();
            if !iter.is_done() {break}
	}
	if let Some(item) = self.it.next() {
            self.j = Some((self.f)(item));
	} else {break}
    }}
    #[inline]fn is_done(&self) -> bool {
        match self.j {Some(ref iter) => iter.is_done(),None => true}
    }
    #[inline]fn get(&self) -> Option<&Self::Item> {
        self.j.as_ref().and_then(J::get)
    }
}

impl<I: StreamingIterator> Streaming for I {}
/// Iterator adapter to iterate over combinations of items from the same iterator.
///
/// see [`Streaming::inner_combinations`].
#[derive(Debug,Clone)]
pub struct InnerCombinations<T,F,S> {
    f: F,
    target: T,
    stack: S,
}
impl<I,T,F,S> StreamingIterator for InnerCombinations<T,F,S>
where I: StreamingIterator+Clone, F: Fn(&I::Item) -> T, S: Stack<Item=I>,
      T: AddAssign<T> + SubAssign<T> + Ord + Clone {
    type Item = S;
    fn advance(&mut self) {
	while !(self.stack.is_empty()) {
	    let last_len = (self.f)(self.stack.get().unwrap().get().unwrap());
	    match last_len.cmp(&self.target) {
		Ordering::Greater => {},
		order => {
		    if self.stack.try_push(self.stack.get().unwrap().clone()) {
			self.target -= last_len;
			if order.is_eq() {return} else {continue}
		    }
		},
	    }
	    loop {
		let last_len = (self.f)(self.stack.get().unwrap().get().unwrap());
		self.target += last_len;
		let new_len = self.stack.get_mut().unwrap().next().map(&self.f);
		match new_len.as_ref().map_or(Ordering::Greater,
					       |l| l.cmp(&self.target)) {
		    Ordering::Greater => {
			self.stack.pop();
			if self.stack.is_empty() {return}
		    },
		    order => {
			self.target -= new_len.unwrap();
			if order.is_eq() {return} else {break}
		    }
		    
		}
	    }
	}
    }
    fn get(&self) -> Option<&Self::Item> {
	if self.stack.is_empty() {None} else {
	    Some(&self.stack)
	}
    }
}
/// A trait defining a Stack datastructure.
///
/// Any such datastructure can be used to store the iterators for [`Streaming::inner_combinations`]. Is implemented for [`Vec`]
pub trait Stack {
    /// The type of items the stack holds
    type Item;
    /// try to push to the stack, indicate success. This may fail, e.g. for a
    /// limited buffer.
    fn try_push(&mut self, elem: Self::Item) -> bool;
    /// pop from the stack.
    fn pop(&mut self) -> Option<Self::Item>;
    /// get a reference to the top element
    fn get(&self) -> Option<&Self::Item>;
    /// get a mutable reference to the top element
    fn get_mut(&mut self) -> Option<&mut Self::Item>;
    /// indicate if the stack is empty.
    ///
    /// [`Stack::pop`], [`Stack::get`] and [`Stack::get_mut`] should return [`None`]  **exactly if** this returns [`true`].
    fn is_empty(&self) -> bool;
}
impl<T> Stack for Vec<T> {
    type Item=T;
    fn try_push(&mut self, elem: T) -> bool {self.push(elem);true}
    fn pop(&mut self) -> Option<T> {self.pop()}
    fn get(&self) -> Option<&T> {self.last()}
    fn get_mut(&mut self) -> Option<&mut T> {self.last_mut()}
    fn is_empty(&self) -> bool {self.is_empty()}
}

