
# Specify the base image
FROM gcc:8.5.0

# Do a few updates of the base system
RUN apt-get update && \
        apt-get install -y libxcursor-dev && \
        apt-get install -y libxfixes-dev && \
        apt-get install -y libxft-dev && \
        apt-get install -y libxinerama-dev && \
        apt-get install -y libsdl-pango-dev && \
        apt-get install -y libpango-1.0 && \
        apt-get install -y libpangoxft-1.0 && \
        apt-get install -y libpangocairo-1.0
