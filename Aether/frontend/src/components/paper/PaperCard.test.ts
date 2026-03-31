import { mount } from '@vue/test-utils';
import PaperCard from './PaperCard.vue';

function mountCard(paper: Record<string, unknown>) {
  return mount(PaperCard, {
    props: {
      paper: {
        title: 'Test Paper',
        authors: ['Alice'],
        abstract_text: 'Short abstract',
        url: 'https://example.com',
        is_read: false,
        ...paper,
      },
    },
  });
}

describe('PaperCard', () => {
  it('marks arxiv content as preprint', () => {
    const wrapper = mountCard({
      publication: 'cs.CR',
      url: 'https://arxiv.org/abs/1234.5678',
    });

    expect(wrapper.text()).toContain('Preprint');
  });

  it('marks dblp and conference sources as conference', () => {
    const wrapper = mountCard({
      publication: 'USENIX Security',
      url: 'https://dblp.org/rec/conf/uss/example',
    });

    expect(wrapper.text()).toContain('Conference');
  });

  it('marks non-paper sources as news or blog', () => {
    const wrapper = mountCard({
      publication: 'The Hacker News',
      url: 'https://thehackernews.com/example',
    });

    expect(wrapper.text()).toContain('News / Blog');
  });
});
