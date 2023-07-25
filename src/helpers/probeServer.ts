import polka from 'polka';
import { database } from './database';
import type { PrismaClient } from '@prisma/client';
import type { Client } from 'discord.js';

class ProbeServer {
  server: polka.Polka;
  ready: boolean;
  live: boolean;
  prismaClient: PrismaClient;
  discordClient: Client | null;

  constructor() {
    this.server = polka();
    this.ready = false;
    this.live = false;
    this.prismaClient = database;
    this.discordClient = null;
  }

  init(this: ProbeServer, discordClient: Client) {
    this.discordClient = discordClient;

    return this;
  }

  start(this: ProbeServer) {
    this.server
      .get('/live', async (req, res) => {
        await this.verifyLive();

        if (this.ready && this.live) {
          res.writeHead(200);
          res.end();
        } else {
          res.writeHead(503);
          res.end();
        }
      })
      .get('/ready', (req, res) => {
        if (this.ready) {
          res.writeHead(200);
          res.end();
        } else {
          res.writeHead(503);
          res.end();
        }
      })
      .listen(8080, (err: Error) => {
        if (err) throw err;
        console.info(`[INFO] Probe server listening on 8080`);
      });
  }

  public setReady(this: ProbeServer, ready: boolean) {
    this.ready = ready;
  }

  // We only want to expose a public method to set the live state to false
  public notLive(this: ProbeServer) {
    this.setLive(false);
  }

  private setLive(this: ProbeServer, live: boolean) {
    this.live = live;
  }

  // Check Discord connection and prismadb connection
  public async verifyLive(this: ProbeServer) {
    if (this.discordClient === null) {
      throw new Error('Discord client not set');
    }

    // We don't want to verify liveness if the bot isn't ready
    if (!this.ready) {
      return;
    }

    try {
      const discordStatus = this.discordClient.ws.status;
      const prismaStatus = await this.prismaClient.$connect();

      if (discordStatus === 0 && prismaStatus !== null) {
        this.setLive(true);
      } else {
        this.setLive(false);
      }
    } catch (error: any) {
      console.error("[ERROR] " + error);
      this.setLive(false);
    }
  }
}

export const probeServer = new ProbeServer();