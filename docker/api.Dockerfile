FROM rust:1.51.0

# diesel_cliのinstall
# ファイルを変更してもcacheを使えるように、COPYの前に行う
# 参考: https://tech.plaid.co.jp/improve_docker_build_efficiency/#3-cache-
RUN cargo install diesel_cli --version 1.4.1

# dbを待つためにmysqlコマンドをインストール
RUN apt update && \
    apt install -y default-mysql-client && \
    apt search caching-sha2-password

WORKDIR app
COPY . .
COPY ./docker/wait_for_db.sh .

ENV TMP_DIR /tmp

# dockerに依存ライブラリのバイナリをキャッシュさせるよう教える
# これがないとコード変わるたびに毎回ライブラリのインストールから始まるらしい
# 参考: https://qiita.com/_mkazutaka/items/c4b602327c2ff7913718
# 普通に/app/target/にバイナリおくと何故か消える（これクソすぎる）ので、
# 仮ディレクトリにコピーしてから戻す
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,sharing=private,target=/app/target \
    --mount=type=cache,sharing=private,target=$TMP_DIR \
    cargo build --release && \
    mkdir -p $TMP_DIR && \
    cp -r ./target/* $TMP_DIR
# 元の場所に戻す
RUN mv -f $TMP_DIR/* ./target

RUN ./docker/download_srtm_datas.sh
