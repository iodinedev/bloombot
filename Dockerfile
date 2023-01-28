ARG PRODUCTION=false
FROM node:18

# Create app directory
WORKDIR /usr/src/app

# Copy over project source
COPY . .


# Echo the database URL from the environment variable
RUN echo "$DATABASE_URL"
RUN echo "Test"

ENV DATABASE_URL=postgresql://postgres:*WN%to*s8!7BDo@104.248.235.240:5432/bloombot_test
ENV DISCORD_TOKEN=NzEyNjk4NDM0NjcwMjk3MTA4.GudKTt.PWL0_aF05HXiDrdEPmK1Y16EFzin7fJqhxmW-4

# Install package dependencies
RUN yarn install
RUN yarn prisma db pull
RUN yarn prisma generate


RUN rm -rf dist
RUN yarn tsc

CMD [ "node", "dist/bot.js" ]
