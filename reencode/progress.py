import progressbar
import multiprocessing


def start_pbar(queue, live_progress):
    def run_pbar(q):
        pmessage = progressbar.FormatCustomText('File: %(file).50s -- %(status)s', dict(file='none', status='idle'))
        widgets = [
            progressbar.AnimatedMarker(),
            ' :: ', pmessage, ' :: ',
            progressbar.Timer(),
        ]
        pbar = progressbar.ProgressBar(widgets=widgets, max_value=progressbar.UnknownLength)
        while True:
            if live_progress:
                pbar.update()
            try:
                message = q.get(True, 1)
                if message == 'next':
                    pbar.finish()
                    pmessage.update_mapping(file='none', status='idle')
                    pbar = progressbar.ProgressBar(widgets=widgets, max_value=progressbar.UnknownLength)
                else:
                    pmessage.update_mapping(**message)
            except Exception:
                pass
    process = multiprocessing.Process(target=run_pbar, args=(queue,), daemon=True)
    process.start()
