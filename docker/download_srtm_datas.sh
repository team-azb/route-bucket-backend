mkdir -p srtm_data
cd srtm_data \
&& wget https://srtm.csi.cgiar.org/wp-content/uploads/files/srtm_30x30/TIFF/N30E120.zip \
    -O srtm.zip \
&& unzip srtm.zip \
&& mv "$(unzip -Z1 srtm.zip)" srtm.tif \
&& rm srtm.zip