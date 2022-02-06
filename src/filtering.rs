use crate::sample::Sample;

pub struct FilteringContext {
    nom: usize,
    denom: usize,
    input_step_low: usize,
    step_acc: usize,
    filter_index: usize,
    pub filter_length: usize,
    filters: Vec<f32>, /* All filters consecutively, lenght must be
                     filter_length * filter_count*/
    pub channels: usize,
    input_start: usize, // Where to start processing next input block
}

impl FilteringContext
{
    pub fn new(
        filters: Vec<f32>,
        filter_length: usize,
        nom: usize,
        denom: usize,
        channels: usize,
    ) -> FilteringContext {
        let input_step_low = (nom / denom) * channels;
        let nom = nom % denom;

        FilteringContext {
            nom,
            denom,
            input_step_low,
            step_acc: 0,
            filter_index: 0,
            filter_length,
            filters,
            channels,
            input_start: 0,
        }
    }
    pub fn apply_filters<I,O>(&mut self, input: &[I], output: &mut [O]) -> usize
    where I: Sample + Copy, O: Sample{
	
        let mut input_index = self.input_start;
        let mut output_index = 0;
        let filters = &self.filters;
        let mut filter_index = self.filter_index;
        let filter_length = self.filter_length;
        let input_step_low = self.input_step_low;
        let mut step_acc = self.step_acc;
        let nom = self.nom;
        let denom = self.denom;
        let channels = self.channels;
        let input_length = input.len() - (filter_length - 1) * channels;
        while input_index < input_length {
            //println!("{} -> {}", input_index, output_index);
            for c in 0..channels {
                let mut filter_index = filter_index;
                let mut input_index = input_index + c;
                let mut acc: f32 = 0.0;
                for _ in 0..filter_length {
                    acc += filters[filter_index] * input[input_index].normailze();
                    filter_index += 1;
                    input_index += channels;
                }
                output[output_index] =  O::full(acc);
                output_index += 1;
            }
            filter_index += filter_length;
            if filter_index == filters.len() {
                filter_index = 0;
            }
            input_index += input_step_low;
            step_acc += nom;
            if step_acc >= denom {
                step_acc -= denom;
                input_index += channels;
            }
        }
        self.filter_index = filter_index;
        self.step_acc = step_acc;
        self.input_start = input_index - input_length;
        output_index
    }
}

#[test]
fn test_stepping() {
    let unit_filter = vec![1.0f32];
    let mut filter = FilteringContext::new(unit_filter.clone(), 1, 6, 4, 1);
    let input = [
        1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0,
    ];
    let mut output = [-1.0; 10];
    filter.apply_filters(&input, &mut output);
    assert_eq!(
        output,
        [1.0, 2.0, 4.0, 5.0, 7.0, 8.0, 10.0, 11.0, 13.0, 14.0]
    );

    let mut filter = FilteringContext::new(unit_filter.clone(), 1, 6, 4, 2);
    let mut output = [-1.0; 10];
    filter.apply_filters(&input, &mut output);
    assert_eq!(
        output,
        [1.0, 2.0, 3.0, 4.0, 7.0, 8.0, 9.0, 10.0, 13.0, 14.0]
    );

    let mut filter = FilteringContext::new(unit_filter, 1, 4, 6, 2);
    let mut output = [-1.0; 22];
    filter.apply_filters(&input, &mut output);
    assert_eq!(
        output,
        [
            1.0, 2.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 9.0, 10.0, 11.0,
            12.0, 13.0, 14.0, 13.0, 14.0
        ]
    );
}
