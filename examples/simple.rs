extern crate genevofra;
extern crate rand;

use genevofra::*;
use rand::Rng;


fn main()
{
	let mut opt = Optimizer::new();
	opt.add_item(Example::new());
	
	for i in 0..10
	{
		let n = 1;
		let score = opt.optimize(n);
		println!("Score after {} iterations: {}", (i + 1) * n, score);
	}
}


#[derive(Clone)]
struct Example
{
	fct:f64,
}

impl Example
{
	pub fn new() -> Example
	{
		let mut rng = rand::thread_rng();
		let number = rng.gen::<f64>() * 1000.0;
		Example { fct: number }
	}
}

impl GEF for Example
{
	fn breed(&self, other:&Self) -> Example
	{
		let mut rng = rand::thread_rng();
		//randomly choose between average and selection
		if rng.gen::<f64>() < 0.5
		{ //average
			let avg = (self.fct + other.fct) / 2.0;
			Example { fct: avg }
		}
		else
		{ //select (normally selection is more useful, because more variables are chosen randomly, so that it mixes up things)
			if rng.gen::<f64>() < 0.5
			{
				self.clone()
			}
			else
			{
				other.clone()
			}
		}
	}
	
	fn mutate(&mut self)
	{
		let mut rng = rand::thread_rng();
		//modify
		let delta = rng.gen::<f64>() * 100.0 - 50.0;
		self.fct += delta;
		//randomly renew with 5% probabiliy
		if rng.gen::<f64>() < 0.05
		{
			self.fct = rng.gen::<f64>() * 1000.0;
		}
	}
	
	//evaluate as inverted squared error to target value (we want to minimize instead of maximize)
	fn evaluate(&self) -> f64
	{
		let target = 125.0;
		let error = self.fct - target;
		
		-error * error
	}
}

