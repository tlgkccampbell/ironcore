$command = "cmd.exe /C cargo build --manifest-path projects/ironcore-native/Cargo.toml"
Invoke-Expression -Command:$command
$command = "cmd.exe /C cargo build --manifest-path projects/ironcore-host/Cargo.toml"
Invoke-Expression -Command:$command