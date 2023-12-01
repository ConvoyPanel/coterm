FROM rust:1.69

RUN apt-get update
RUN apt-get -y install software-properties-common curl sudo unzip

RUN curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
RUN apt-get install -y nodejs