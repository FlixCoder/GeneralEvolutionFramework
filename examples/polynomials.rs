extern crate genevofra;
extern crate rand;

use genevofra::*;
use rand::Rng;

const PROB_AVG:f64 = 0.1; //probability to average factors instead of selecting
const PROB_DEGREE:f64 = 0.01; //probability to increase the degree of a polynomial function by 1
const PROB_NEW:f64 = 0.05; //for every value: probability to set to random value
const RANGE_NEW:f64 = 20.0; //size of interval to create random factors
const RANGE_MOD:f64 = 2.0; //size of interval to modify factors

const REGULARIZATION:f64 = 0.5; //factor for additional error term (multiplied to degree)


//optimizing polynomial functions to fit to the points
fn main()
{
	let points = vec![	(0.0, 0.1),
						(1.0, 1.4),
						(2.0, 4.0),
						(3.0, 8.1),
						(4.0, 16.5)
					];
	
	let mut opt = Optimizer::new();
	opt.set_population(100)
		.set_survive(8)
		.set_bad_survive(2)
		.set_prob_mutate(0.9)
		.add_item(Polynome::new(points));
	
	//train
	for i in 0..10
	{
		let n = 20;
		let score = opt.optimize(n);
		println!("Score after {} iterations: {}", (i + 1) * n, score);
	}
	
	//output function
	let fct = opt.get_best_ref();
	fct.print();
}


fn calc_f(factors:&Vec<f64>, x:f64) -> f64
{
	let mut result = factors[0];
	let mut current_pow = 1.0;
	for i in 1..factors.len()
	{
		current_pow *= x;
		result += factors[i] * current_pow;
	}
	result
}


#[derive(Clone)]
struct Polynome
{
	fct:Vec<f64>, //factors accordingly for index => x^index
	target:Vec<(f64,f64)>, //points that should be matched by the polynomial function
}

impl Polynome
{
	pub fn new(points:Vec<(f64,f64)>) -> Polynome
	{
		let mut vec = Vec::new();
		vec.push(0.0); //constant zero function with degree 0
		Polynome { fct: vec, target: points }
	}
	
	pub fn print(&self)
	{
		let mut str = String::new();
		for i in 0..self.fct.len()
		{
			str.push_str(&format!("{}x^{} + ", self.fct[i], i));
		}
		let tmp = str.len() - 3;
		str.truncate(tmp);
		println!("{}", str);
	}
}

impl GEF for Polynome
{
	fn breed(&self, other:&Self) -> Polynome
	{
		let mut rng = rand::thread_rng();
		let mut new = self.clone();
		//randomly choose between average and selection
		if rng.gen::<f64>() < PROB_AVG
		{ //average
			for i in 0..new.fct.len()
			{
				if other.fct.len() <= i { break; }
				new.fct[i] = (new.fct[i] + other.fct[i]) / 2.0;
			}
		}
		else
		{ //select (normally selection is more useful, because more variables are chosen randomly, so that it mixes up things)
			for i in 0..new.fct.len()
			{
				if other.fct.len() <= i { break; }
				if rng.gen::<f64>() < 0.5
				{ //select value from other randomly
					new.fct[i] = other.fct[i];
				} //else is not necessary, already set
			}
		}
		new
	}
	
	fn mutate(&mut self)
	{
		let mut rng = rand::thread_rng();
		
		if rng.gen::<f64>() < PROB_DEGREE
		{ //increase polynomial degree
			self.fct.push(0.0);
		}
		
		for i in 0..self.fct.len()
		{
			if rng.gen::<f64>() < PROB_NEW
			{ //set to random
				self.fct[i] = rng.gen::<f64>() * RANGE_NEW - RANGE_NEW / 2.0;
			}
			else
			{ //modify randomly
				let delta = rng.gen::<f64>() * RANGE_MOD - RANGE_MOD / 2.0;
				self.fct[i] += delta;
			}
		}
	}
	
	//evaluate as inverted mean squared error to target (we want to minimize instead of maximize)
	fn evaluate(&self) -> f64
	{
		//calculate mean squared error //TODO: maybe use relative error instead of absolute
		let mut mse = 0.0;
		for point in &self.target
		{
			let error = point.1 - calc_f(&self.fct, point.0);
			mse += error * error;
		}
		mse /= self.target.len() as f64;
		
		//add regularization
		let reg = REGULARIZATION * (self.fct.len() -1) as f64; //regularization
		
		//return
		-(mse + reg)
	}
}

