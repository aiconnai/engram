import { describe, it, expect, vi, beforeEach } from 'vitest';
import { EngramClient } from './index';

// Mock global fetch
const mockFetch = vi.fn();
vi.stubGlobal('fetch', mockFetch);

describe('EngramClient', () => {
  let client: EngramClient;
  const config = {
    baseUrl: 'https://test.engram.dev',
    apiKey: 'test-key',
    tenant: 'test-tenant',
    timeout: 5000,
  };

  beforeEach(() => {
    client = new EngramClient(config);
    mockFetch.mockClear();
  });

  describe('constructor', () => {
    it('should store config values', () => {
      expect(client).toBeDefined();
    });

    it('should strip trailing slash from baseUrl', () => {
      const c = new EngramClient({
        ...config,
        baseUrl: 'https://test.engram.dev/',
      });
      // Access private property via type assertion for testing
      expect((c as any).config.baseUrl).toBe('https://test.engram.dev');
    });
  });

  describe('mcpCall', () => {
    it('should make POST request with correct headers', async () => {
      const mockResponse = {
        ok: true,
        json: () => Promise.resolve({
          jsonrpc: '2.0',
          id: 1,
          result: { id: 123, content: 'Test' },
        }),
      };
      mockFetch.mockResolvedValueOnce(mockResponse);

      // Access private method via type assertion
      const result = await (client as any).mcpCall('memory_create', { content: 'test' });

      expect(mockFetch).toHaveBeenCalledWith(
        'https://test.engram.dev/v1/mcp',
        expect.objectContaining({
          method: 'POST',
          headers: expect.objectContaining({
            'Authorization': 'Bearer test-key',
            'X-Tenant-Slug': 'test-tenant',
            'Content-Type': 'application/json',
          }),
        })
      );
      expect(result).toEqual({ id: 123, content: 'Test' });
    });

    it('should handle HTTP errors', async () => {
      const mockResponse = {
        ok: false,
        status: 404,
        text: () => Promise.resolve('Not Found'),
      };
      mockFetch.mockResolvedValueOnce(mockResponse);

      await expect((client as any).mcpCall('memory_get', { id: 999 }))
        .rejects.toThrow('HTTP 404');
    });

    it('should handle JSON-RPC errors', async () => {
      const mockResponse = {
        ok: true,
        json: () => Promise.resolve({
          jsonrpc: '2.0',
          id: 1,
          error: { message: 'Invalid params', code: -32602 },
        }),
      };
      mockFetch.mockResolvedValueOnce(mockResponse);

      await expect((client as any).mcpCall('memory_get', { id: 'invalid' }))
        .rejects.toThrow('Invalid params');
    });

    it('should increment request IDs', async () => {
      const mockResponse = {
        ok: true,
        json: () => Promise.resolve({ jsonrpc: '2.0', id: 1, result: {} }),
      };
      mockFetch.mockResolvedValue(mockResponse);

      await (client as any).mcpCall('test', {});
      await (client as any).mcpCall('test', {});
      await (client as any).mcpCall('test', {});

      const calls = mockFetch.mock.calls;
      const ids = calls.map((call: any) => JSON.parse(call[1].body).id);
      expect(ids).toEqual([1, 2, 3]);
    });
  });

  describe('create', () => {
    it('should call mcpCall with correct params', async () => {
      const mockResponse = {
        ok: true,
        json: () => Promise.resolve({
          jsonrpc: '2.0',
          id: 1,
          result: { id: 123 },
        }),
      };
      mockFetch.mockResolvedValueOnce(mockResponse);

      const result = await client.create('Hello world');

      const callBody = JSON.parse(mockFetch.mock.calls[0][1].body);
      expect(callBody.params.arguments.content).toBe('Hello world');
      expect(callBody.params.arguments.memory_type).toBe('note'); // default
      expect(result).toEqual({ id: 123 });
    });

    it('should pass all optional params', async () => {
      const mockResponse = {
        ok: true,
        json: () => Promise.resolve({ jsonrpc: '2.0', id: 1, result: {} }),
      };
      mockFetch.mockResolvedValueOnce(mockResponse);

      await client.create(
        'Test content',
        'image',
        ['tag1', 'tag2'],
        'my-workspace',
        { source: 'test' },
        0.8,
        'https://example.com/img.jpg'
      );

      const args = JSON.parse(mockFetch.mock.calls[0][1].body).params.arguments;
      expect(args.content).toBe('Test content');
      expect(args.memory_type).toBe('image');
      expect(args.tags).toEqual(['tag1', 'tag2']);
      expect(args.workspace).toBe('my-workspace');
      expect(args.metadata).toEqual({ source: 'test' });
      expect(args.importance).toBe(0.8);
      expect(args.media_url).toBe('https://example.com/img.jpg');
    });
  });

  describe('list', () => {
    it('should use default params', async () => {
      const mockResponse = {
        ok: true,
        json: () => Promise.resolve({ jsonrpc: '2.0', id: 1, result: {} }),
      };
      mockFetch.mockResolvedValueOnce(mockResponse);

      await client.list();

      const args = JSON.parse(mockFetch.mock.calls[0][1].body).params.arguments;
      expect(args.limit).toBe(50);
      expect(args.offset).toBe(0);
    });

    it('should map filter_ to filter in API call', async () => {
      const mockResponse = {
        ok: true,
        json: () => Promise.resolve({ jsonrpc: '2.0', id: 1, result: {} }),
      };
      mockFetch.mockResolvedValueOnce(mockResponse);

      const filter = { field: 'value' };
      await client.list(undefined, undefined, undefined, undefined, filter);

      const args = JSON.parse(mockFetch.mock.calls[0][1].body).params.arguments;
      expect(args.filter).toEqual(filter); // Mapped to "filter" for API
    });
  });

  describe('search', () => {
    it('should call with query and default limit', async () => {
      const mockResponse = {
        ok: true,
        json: () => Promise.resolve({ jsonrpc: '2.0', id: 1, result: {} }),
      };
      mockFetch.mockResolvedValueOnce(mockResponse);

      await client.search('test query');

      const args = JSON.parse(mockFetch.mock.calls[0][1].body).params.arguments;
      expect(args.query).toBe('test query');
      expect(args.limit).toBe(10);
    });
  });

  describe('get/update/delete', () => {
    it('should get memory by id', async () => {
      const mockResponse = {
        ok: true,
        json: () => Promise.resolve({ jsonrpc: '2.0', id: 1, result: {} }),
      };
      mockFetch.mockResolvedValueOnce(mockResponse);

      await client.get(123);

      const args = JSON.parse(mockFetch.mock.calls[0][1].body).params.arguments;
      expect(args.id).toBe(123);
    });

    it('should update memory', async () => {
      const mockResponse = {
        ok: true,
        json: () => Promise.resolve({ jsonrpc: '2.0', id: 1, result: {} }),
      };
      mockFetch.mockResolvedValueOnce(mockResponse);

      await client.update(123, 'Updated content');

      const args = JSON.parse(mockFetch.mock.calls[0][1].body).params.arguments;
      expect(args.id).toBe(123);
      expect(args.content).toBe('Updated content');
    });

    it('should delete memory', async () => {
      const mockResponse = {
        ok: true,
        json: () => Promise.resolve({ jsonrpc: '2.0', id: 1, result: {} }),
      };
      mockFetch.mockResolvedValueOnce(mockResponse);

      await client.delete(123);

      const args = JSON.parse(mockFetch.mock.calls[0][1].body).params.arguments;
      expect(args.id).toBe(123);
    });
  });
});
