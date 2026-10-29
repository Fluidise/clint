FROM node:26.13.0-bookworm-slim@sha256:bc0a2afd3b74ce12222fc8bab99271e66257f021078f5f4a5d687a77d280c1d9 AS node
FROM rust:1.99.1-slim-bookworm@sha256:24707e70a4f1e23d9a7b25202ec43126d28a52294b96aad5c670df1fd1f1d400
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
