FROM rust

COPY target/release/chola /bin/chola

CMD ["bin/chola"]