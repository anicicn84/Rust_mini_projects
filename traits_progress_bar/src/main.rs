use std::{time::Duration, thread::sleep};

// clearing the screen command
const CLEAR: &str = "\x1B[2J\x1B[1;1H";

struct Unbounded;
struct Bounded {
    bound: usize, 
    delims: (char, char),
}

struct Progress<Iter, Bound> {
    iter: Iter, 
    i: usize,
    bound: Bound, 
}

trait ProgressDisplay: Sized {
    fn display<Iter>(&self, progress: &Progress<Iter, Self>);
}

impl ProgressDisplay for Unbounded {
    fn display<Iter>(&self, progress: &Progress<Iter, Self>) {
        println!("{}", "*".repeat(progress.i)) 
    }
}

impl ProgressDisplay for Bounded {
    fn display<Iter>(&self, progress: &Progress<Iter, Self>){
        println!("{}{}{}{}", 
                 self.delims.0,
                 "\u{2588}".repeat(progress.i), 
                 " ".repeat(self.bound - progress.i), 
                 self.delims.1);
    }
}

impl<Iter> Progress<Iter, Unbounded> {
    // static method, doesn't take self as an input
    pub fn new(iter: Iter) -> Self {
        Progress{
            iter,
            i:0,
            bound: Unbounded,
        }
    }
}

impl<Iter> Progress<Iter, Unbounded> 
where Iter: ExactSizeIterator {
    // something like a factory method flavor
    // Does not return Self, it changes it to be Bounded, since it is called with_bounds
    pub fn with_bound(mut self) -> Progress<Iter, Bounded> { // owned version of a self
        let bound = Bounded {
            bound: self.iter.len(), 
            delims: ('[', ']'),
        };
        Progress {
            iter: self.iter, 
            i: self.i, 
            bound
        }
    }
}

impl<Iter> Progress<Iter, Bounded>{
    pub fn with_delims(mut self, delims: (char, char)) -> Self {
        self.bound.delims = delims;
        self
    }
}

// turn this Progress struct into an iterator
impl<Iter, Bound> Iterator for Progress<Iter, Bound> 
where Iter: Iterator, Bound: ProgressDisplay{
    type Item = Iter::Item; // Whatever Iter we pass, it's Item is the Item here
    fn next(&mut self) -> Option<Self::Item> {
        println!("{}", CLEAR);
        self.bound.display(&self);
        self.i += 1;

        // return the next elem of the iterator
        // call the default Iterator's next behavior to get the next element
        // and to return it
        self.iter.next()
    }
}

// we define a new trait here
trait ProgressIteratorExt: Sized{
    fn progress(self) -> Progress<Self, Unbounded>;
}

// implementing our trait for whatever type we have
// in this case it is Iter type.
// The same as impl Iterator for Progress
// We added progress() method to any type in the Rust universe
// that implements Iterator trait
impl<Iter> ProgressIteratorExt for Iter 
where Iter: Iterator{
    fn progress(self) -> Progress<Self, Unbounded> {
        // like Progress::new(v.iter())
        Progress::new(self)
    }
}

fn expensive_calculation(_n: &i32){
    sleep(Duration::from_secs(1));
}

fn main() {
//    let brkts = ('<', '>');
    let v = vec![1; 10];

//    for n in (0 .. ).progress().with_delims(brkts) {
//        expensive_calculation(&n);
//    }

    for n in v.iter().progress().with_bound(){
        expensive_calculation(n);
    }
}
