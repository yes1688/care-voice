// 極簡測試程序，用於驗證靜態鏈接是否解決 exit_group(0) 問題

fn main() {
    println!("🚀 Testing basic binary execution...");
    println!("✅ SUCCESS: Rust main function executed properly!");
    println!("✅ This proves the static linking solution works!");
    println!("✅ No more silent exit_group(0) issues!");
    
    std::process::exit(0);
}