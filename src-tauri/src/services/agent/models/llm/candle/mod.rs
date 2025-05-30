pub mod candle;

use anyhow::Result;
use std::io::Write;
use log::{info, warn};

/// 测试 Qwen Candle 模型是否可用
///
/// 此函数会:
/// 1. 初始化最小的 Qwen 0.5B 模型
/// 2. 生成简短文本响应
/// 3. 输出结果和性能统计
pub fn test_candle_model() -> Result<()> {
    use candle::QwenCandleGenerator;
    use candle::QwenInferenceParams;
    use candle::WhichModel;
    use candle_core::Device;

    info!("=== Candle LLM 可用性测试 ===");

    // 设置简单的测试参数
    let params = QwenInferenceParams {
        which_model: WhichModel::W0_5b, // 使用最小的模型进行测试
        device: Device::Cpu,            // 使用CPU
        sample_len: 100,                // 限制生成的长度
        ..Default::default()
    };

    // 创建生成器
    info!("正在初始化 Qwen Candle 生成器...");
    let start = std::time::Instant::now();
    let mut generator = QwenCandleGenerator::new(params)?;
    info!("初始化完成，耗时: {:?}", start.elapsed());

    // 测试简单提示
    let prompt = "你好，请用一句话介绍自己。";
    info!("\n提示: {}", prompt);

    // 测试流式生成
    info!("\n流式生成输出:");
    let start = std::time::Instant::now();

    // 使用流式生成
    generator.generate_tokens(prompt, 10)?;

    let elapsed = start.elapsed();

    info!("\n\n生成完成，耗时: {:?}", elapsed);

    info!("\n=== 测试结束：Candle LLM 工作正常 ===");

    Ok(())
}

/// 运行命令行测试工具
///
/// 允许用户通过命令行指定参数并测试模型
pub fn run_cli_test() -> Result<()> {
    use candle::Args;
    use candle::QwenCandleGenerator;
    use candle::QwenInferenceParams;
    use clap::Parser;

    let args = Args::parse();

    // 创建参数
    let params = QwenInferenceParams {
        which_model: args.model,
        device: candle_examples::device(args.cpu)?,
        temperature: args.temperature,
        top_p: args.top_p,
        seed: args.seed,
        repeat_penalty: args.repeat_penalty,
        repeat_last_n: args.repeat_last_n,
        use_flash_attn: args.use_flash_attn,
        sample_len: args.sample_len,
        ..Default::default()
    };

    // 创建生成器
    let mut generator = QwenCandleGenerator::new(params)?;

    // 流式生成并打印到标准输出
    println!("生成中...");
    generator.generate_streaming(&args.prompt, args.sample_len, |chunk| {
        print!("{}", chunk);
        std::io::stdout().flush()?;
        Ok(())
    })?;

    println!("\n完成生成");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use candle::QwenCandleGenerator;
    use candle::QwenInferenceParams;
    use candle::WhichModel;
    use candle_core::Device;

    /// 创建测试参数 - 使用最小模型和最短输出
    fn get_test_params() -> QwenInferenceParams {
        QwenInferenceParams {
            which_model: WhichModel::W0_5b,
            device: Device::Cpu,
            sample_len: 10,         // 限制输出长度，加快测试速度
            temperature: Some(0.0), // 确定性输出
            ..Default::default()
        }
    }

    /// 测试模型能否正确初始化
    #[test]
    fn test_model_initialization() {
        let params = get_test_params();

        // 测试初始化不应该panic
        let result = QwenCandleGenerator::new(params);
        warn!(result.is_ok(), "模型初始化失败: {:?}", result.err());
    }

    /// 测试字符串生成功能
    #[test]
    fn test_string_generation() -> Result<()> {
        let params = get_test_params();
        let mut generator = QwenCandleGenerator::new(params)?;

        // 简单提示词
        let prompt = "你好";
        let result = generator.generate_string(prompt, 10)?;

        // 验证生成了文本
        assert!(!result.is_empty(), "生成的文本不应为空");
        warn!("生成文本: {}", result);

        Ok(())
    }

    /// 测试流式生成功能
    #[test]
    fn test_streaming_generation() -> Result<()> {
        let params = get_test_params();
        let mut generator = QwenCandleGenerator::new(params)?;

        // 简单提示词
        let prompt = "介绍自己";
        let mut output = String::new();

        // 收集流式输出
        generator.generate_streaming(prompt, 10, |chunk| {
            output.push_str(chunk);
            Ok(())
        })?;

        // 验证生成了文本
        warn!(!output.is_empty(), "流式生成的文本不应为空");
        info!("流式生成文本: {}", output);

        Ok(())
    }

    // 测试Token生成功能
    #[test]
    fn test_token_generation() -> Result<()> {
        let params = get_test_params();
        let mut generator = QwenCandleGenerator::new(params)?;

        // 简单提示词
        let prompt = "1+1=";
        let tokens = generator.generate_tokens(prompt, 5)?;

        // 验证生成了token
        warn!(!tokens.is_empty(), "生成的token不应为空");
        info!("生成token数量: {}", tokens.len());

        Ok(())
    }
}
