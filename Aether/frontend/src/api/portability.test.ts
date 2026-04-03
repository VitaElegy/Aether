import axios from 'axios';
import { portabilityApi } from './portability';

vi.mock('axios', () => ({
  default: {
    get: vi.fn(),
    post: vi.fn(),
  },
}));

const axiosMock = vi.mocked(axios, true);

class MockEventSource {
  static instances: MockEventSource[] = [];
  url: string;
  onmessage: ((event: MessageEvent<string>) => void) | null = null;
  onerror: ((event: Event) => void) | null = null;
  close = vi.fn();

  constructor(url: string) {
    this.url = url;
    MockEventSource.instances.push(this);
  }
}

describe('portabilityApi', () => {
  beforeEach(() => {
    localStorage.clear();
    localStorage.setItem('token', 'portability-token');
    MockEventSource.instances = [];
    vi.stubGlobal('EventSource', MockEventSource as unknown as typeof EventSource);
  });

  it('sends auth headers for preview and start export calls', async () => {
    axiosMock.get.mockResolvedValueOnce({ data: { total_items: 1, estimated_size: '1 MB', sections: [] } });
    axiosMock.post.mockResolvedValueOnce({ data: { task_id: 'task-1' } });

    await portabilityApi.analyzeExport('kb-1');
    await portabilityApi.startExport('kb-1');

    expect(axiosMock.get).toHaveBeenCalledWith('/api/portability/kb-1/export/preview', expect.objectContaining({
      headers: expect.objectContaining({
        Authorization: 'Bearer portability-token',
      }),
    }));
    expect(axiosMock.post).toHaveBeenCalledWith('/api/portability/kb-1/export/start', {}, expect.objectContaining({
      headers: expect.objectContaining({
        Authorization: 'Bearer portability-token',
      }),
    }));
  });

  it('parses progress events and closes stream on completion', () => {
    const onEvent = vi.fn();
    const onError = vi.fn();

    portabilityApi.connectProgress('task-9', onEvent, onError);
    const es = MockEventSource.instances[0];

    es.onmessage?.({ data: JSON.stringify({ stage: 'Completed', percent: 100 }) } as MessageEvent<string>);

    expect(es.url).toBe('/api/portability/tasks/task-9/progress');
    expect(onEvent).toHaveBeenCalledWith({ stage: 'Completed', percent: 100 });
    expect(es.close).toHaveBeenCalled();
  });

  it('forwards EventSource errors and closes the stream', () => {
    const onEvent = vi.fn();
    const onError = vi.fn();

    portabilityApi.connectProgress('task-10', onEvent, onError);
    const es = MockEventSource.instances[0];
    const errorEvent = new Event('error');

    es.onerror?.(errorEvent);

    expect(onError).toHaveBeenCalledWith(errorEvent);
    expect(es.close).toHaveBeenCalled();
  });
});
