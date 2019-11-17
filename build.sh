cargo build --release

docker build -t schoolboy/chola-nightly .

docker push schoolboy/chola-nightly