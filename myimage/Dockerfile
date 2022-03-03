
# Specify the base image -- here we're using one that bundles the OpenJDK version of Java 8 on top of a generic Debian Linux OS
FROM gcc:8.5.0

# Do a few updates of the base system and install R (via the r-base package)
RUN apt-get update && \
        apt-get install -y libxcursor-dev && \
        apt-get install -y libxfixes-dev && \
        apt-get install -y libxft-dev && \
        apt-get install -y libxinerama-dev && \
        apt-get install -y libsdl-pango-dev && \
        apt-get install -y libpango-1.0 && \
        apt-get install -y libpangoxft-1.0 && \
        apt-get install -y libpangocairo-1.0
