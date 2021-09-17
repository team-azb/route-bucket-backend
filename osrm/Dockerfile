FROM osrm/osrm-backend:v5.24.0

# 必要コマンドのインストール
RUN apt-get -q update && \
    apt-get -q install -y wget

# 地図情報のダウンロード（ここいじるとダウンロードやり直しになるので注意）
RUN mkdir /data && \
    wget --progress=bar:force:noscroll -O /data/map.osm.pbf https://download.geofabrik.de/asia/japan/kanto-latest.osm.pbf

# 地図データの前処理
COPY ./bicycle.lua /opt/bicycle.lua
RUN osrm-extract -p /opt/bicycle.lua /data/map.osm.pbf && \
    osrm-partition /data/map.osrm && \
    osrm-customize /data/map.osrm
