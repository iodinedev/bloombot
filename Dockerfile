# ---- Base Node ----
FROM node:18 AS base
# Create app directory
WORKDIR /usr/src/app
COPY package.json yarn.lock ./
# Install app dependencies excluding dev dependencies
RUN yarn install --production
# Generate Prisma Client
COPY ./database ./database
RUN yarn prisma generate

# ---- Build ----
FROM base AS build
WORKDIR /usr/src/app
COPY . .
RUN yarn tsc

# --- Release ----
FROM node:18 AS release
# Create app directory
WORKDIR /usr/src/app
# Copy built js code from previous stage
COPY --from=build /usr/src/app/dist ./dist
# Copy node_modules from the base stage
COPY --from=base /usr/src/app/node_modules ./node_modules
COPY package.json yarn.lock ./
CMD [ "node", "dist/bot.js" ]
