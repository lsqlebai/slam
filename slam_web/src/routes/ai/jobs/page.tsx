import { Box } from '@mui/material';
import PageBase from '../../../components/PageBase';
import AIJobList from '../../../components/ai/AIJobList';
import PageHeader from '../../../components/common/PageHeader';
import { TEXTS } from '../../../i18n';
import { useLangStore } from '../../../stores/lang';

function AIJobsInner() {
  const { lang } = useLangStore();
  const text = TEXTS[lang].aiJobs;

  return (
    <Box sx={{ minHeight: '100dvh', bgcolor: 'grey.100' }}>
      <PageHeader headTitle={text.headTitle} title={text.title} />
      <AIJobList lang={lang} />
    </Box>
  );
}

export default function AIJobsPage() {
  return (
    <PageBase>
      <AIJobsInner />
    </PageBase>
  );
}
