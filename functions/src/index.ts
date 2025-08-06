import { onRequest } from "firebase-functions/v2/https";
import { logger } from "firebase-functions";
import { setGlobalOptions } from "firebase-functions/v2";
import axios from "axios";

// Set global options for all functions
setGlobalOptions({
  region: "us-central1",
  maxInstances: 10,
});

// Simple in-memory cache
interface CachedResponse {
  data: any;
  headers: Record<string, string>;
  timestamp: number;
  maxAge: number; // seconds from Cache-Control
}

let cachedResponse: CachedResponse | null = null;
const ORIGINAL_VERSION_URL = "https://cli.nexus.xyz/version.json";

/**
 * Parse max-age from Cache-Control header
 */
function parseMaxAge(cacheControl: string | undefined): number {
  if (!cacheControl) return 0;
  const match = cacheControl.match(/max-age=(\d+)/);
  return match ? parseInt(match[1], 10) : 0;
}

/**
 * Simple caching proxy that respects Cache-Control headers
 */
export const version = onRequest(
  {
    cors: true,
    memory: "256MiB",
    timeoutSeconds: 30,
  },
  async (req, res) => {
    try {
      const now = Math.floor(Date.now() / 1000); // Unix timestamp in seconds
      
      // Check if cached data is still valid
      if (cachedResponse && (now - cachedResponse.timestamp) < cachedResponse.maxAge) {
        logger.info("Serving from cache", {
          cacheAge: now - cachedResponse.timestamp,
          maxAge: cachedResponse.maxAge,
        });
        
        // Just send the data with cache status for debugging
        res.set({
          "Content-Type": "application/json",
          "X-Cache": "HIT",
        });
        
        res.status(200).json(cachedResponse.data);
        return;
      }

      // Cache miss - fetch from origin
      logger.info("Cache miss, fetching from origin");

      try {
        const response = await axios.get(ORIGINAL_VERSION_URL, {
          timeout: 10000,
          headers: {
            "User-Agent": "nexus-version-cache/1.0",
          },
        });

        // Parse cache headers
        const cacheControl = response.headers["cache-control"];
        const maxAge = parseMaxAge(cacheControl);
        
        // Store in cache if max-age > 0
        if (maxAge > 0) {
          cachedResponse = {
            data: response.data,
            headers: {},
            timestamp: now,
            maxAge,
          };
        }

        // Just send the data with cache status
        const responseHeaders = {
          "Content-Type": "application/json",
          "X-Cache": "MISS",
        };

        // Log all headers for debugging
        logger.info("Response headers from origin", {
          headers: response.headers,
          maxAge,
        });

        res.set(responseHeaders);
        res.status(200).json(response.data);
        return;

      } catch (fetchError: any) {
        logger.error("Failed to fetch from origin", {
          error: fetchError.message,
          status: fetchError.response?.status,
        });

        // Serve stale cache if available
        if (cachedResponse) {
          logger.warn("Serving stale cache due to origin error");
          res.set({
            "Content-Type": "application/json",
            "X-Cache": "STALE",
          });
          res.status(200).json(cachedResponse.data);
          return;
        }

        // No cache available
        res.status(503).json({
          error: "Service temporarily unavailable",
          message: "Unable to fetch version information",
        });
        return;
      }

    } catch (error: any) {
      logger.error("Unexpected error", { error: error.message });
      res.status(500).json({
        error: "Internal server error",
        message: "An unexpected error occurred",
      });
    }
  }
); 