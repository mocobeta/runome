from cProfile import Profile
from pstats import Stats
import sys
from runome.tokenizer import Tokenizer

repeat = 10
dump_file = 'runome_tokenizer.profile'
if len(sys.argv) > 1 and sys.argv[1] == '-janome':
    # For comparison with janome
    from janome.tokenizer import Tokenizer as JanomeTokenizer
    t = JanomeTokenizer()
    dump_file = 'janome_tokenizer.profile'
else:
    # Use runome tokenizer
    t = Tokenizer()

with open('text_lemon.txt') as f:
    s = f.read()

profiler = Profile()
profiler.runcall(lambda: [list(t.tokenize(s)) for i in range(repeat)])

stats = Stats(profiler)
stats.strip_dirs()
stats.sort_stats('tottime')
stats.print_stats()
stats.dump_stats(dump_file)
print(f'Result was dumped to {dump_file}.')
