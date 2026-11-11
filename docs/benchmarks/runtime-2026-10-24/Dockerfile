FROM node:26.13.1-bookworm-slim@sha256:5df79989ed685e09f0a98019db4cebfb22fb9df1a53c0161664a011f7ff6b0f1 AS node
FROM rust:1.99.1-slim-bookworm@sha256:f2c004e3d2d8b547e8cf7ffbc29205385f4e71d23e53e4e383065db5e9f822cd
COPY --from=node /usr/local/bin/node /usr/local/bin/node
COPY --from=node /usr/local/lib/node_modules/npm /usr/local/lib/node_modules/npm
RUN ln -s ../lib/node_modules/npm/bin/npm-cli.js /usr/local/bin/npm \
    && apt-get update \
    && apt-get install -y --no-install-recommends git procps python3 python3-venv ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY package.json package-lock.json /opt/commitlint/
RUN npm --prefix /opt/commitlint --registry=https://registry.npmjs.org ci --ignore-scripts --no-audit --no-fund
COPY requirements.txt /opt/requirements.txt
RUN python3 -m venv /opt/commitizen \
    && /opt/commitizen/bin/pip install --disable-pip-version-check --only-binary=:all: --index-url https://pypi.org/simple -r /opt/requirements.txt
ENV PATH="/opt/commitizen/bin:/opt/commitlint/node_modules/.bin:${PATH}" \
    PYTHONDONTWRITEBYTECODE=1
RUN rustup component add rustfmt clippy
WORKDIR /workspace
COPY source/ /workspace/
RUN cargo fetch --locked \
    && cargo build --release --locked \
    && cargo test --locked --all-targets --no-run
COPY run-linux.py /opt/run-linux.py
CMD ["python3", "/opt/run-linux.py"]
