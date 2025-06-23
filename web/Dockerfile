FROM node:22-alpine

WORKDIR /app

ADD package.json package-lock.json /app
RUN npm install

ADD . /app
RUN npm run build

ENTRYPOINT ["node", "/app/.output/server/index.mjs"]