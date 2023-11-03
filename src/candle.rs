use std::{cell::RefCell, error::Error};

use candle_core::{safetensors, DType, Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::{generation::LogitsProcessor, models};
use hf_hub::api::sync::Api;
use tokenizers::Tokenizer;

const DTYPE: DType = DType::F32;

pub struct Candle {
    device: Device,
    config: models::t5::Config,
    tokenizer: Tokenizer,
    model: RefCell<models::t5::T5ForConditionalGeneration>,
}

impl Candle {
    pub fn new(huggingface_model: String) -> Self {
        let device = Device::Cpu;
        let api = Api::new().unwrap();
        let repo = api.model(huggingface_model);

        let tokenizer = Tokenizer::from_file(repo.get("tokenizer.json").unwrap()).unwrap();

        let config = std::fs::read_to_string(repo.get("config.json").unwrap()).unwrap();
        let config: models::t5::Config = serde_json::from_str(&config).unwrap();

        let weights = safetensors::load(repo.get("model.safetensors").unwrap(), &device).unwrap();
        let vb: VarBuilder = VarBuilder::from_tensors(weights, DTYPE, &device);
        let mut model =
            RefCell::new(models::t5::T5ForConditionalGeneration::load(vb, &config).unwrap());

        Self {
            device,
            config,
            tokenizer,
            model,
        }
    }

    pub fn call(self, prompt: String) -> Result<String, Box<dyn Error>> {
        let tokens = self
            .tokenizer
            .encode(prompt, true)
            .unwrap()
            .get_ids()
            .to_vec();
        let input_token_ids = Tensor::new(&tokens[..], &self.device)?.unsqueeze(0)?;

        let mut output_token_ids = [self.config.pad_token_id as u32].to_vec();
        let mut logits_processor = LogitsProcessor::new(299792458, Some(0.8), None);

        let mut model = self.model.borrow_mut();
        let encoder_output = model.encode(&input_token_ids).unwrap();

        let start = std::time::Instant::now();
        let mut result = String::from("");

        for index in 0.. {
            if output_token_ids.len() > 512 {
                break;
            }
            let decoder_token_ids = if index == 0 {
                Tensor::new(output_token_ids.as_slice(), &self.device)?.unsqueeze(0)?
            } else {
                let last_token = *output_token_ids.last().unwrap();
                Tensor::new(&[last_token], &self.device)?.unsqueeze(0)?
            };
            let logits = model
                .decode(&decoder_token_ids, &encoder_output)?
                .squeeze(0)?;

            let start_at = output_token_ids.len().saturating_sub(64);
            let logits = candle_transformers::utils::apply_repeat_penalty(
                &logits,
                1.1,
                &output_token_ids[start_at..],
            )
            .unwrap();

            let next_token_id = logits_processor.sample(&logits).unwrap();
            if next_token_id as usize == self.config.eos_token_id {
                break;
            }
            output_token_ids.push(next_token_id);
            if let Some(text) = self.tokenizer.id_to_token(next_token_id) {
                let text = text.replace('▁', " ").replace("<0x0A>", "\n");
                result += &text;
            }
        }
        let dt = start.elapsed();

        print!("\nElapsed: {}\n", dt.as_secs());

        Ok(result)
    }

    // let tokenizer = tokenizer.with_padding(None).with_truncation(None).unwrap();
}
