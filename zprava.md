<link rel="stylesheet" type="text/css" href="pandoc.css"/>

Problém batohu
==============

Definice problému
-----------------

Je daná množina předmětů s váhou a cenou. Úkolem je najít takovou podmnožinu, aby součet váh byl do maximální povolené váhy a zároveň celková cena podmnožiny byla alespoň taková, jaká je zadaná na vstupu (rozhodovací problém).

Implementace
------------

Program je napsán v jazyce Rust. To je relativně nový jazyk. V rychlosti se mu daří konkurovat nízkoúrovnovým jazykům (ve většině benchmarcích vychází o trošku hůře než C++), ale píše se v něm spíše tak, že se to podobá vysokoúrovnovým. Má propracovaný pamětový model, a tak je to jeden z mála jazyků, který je pamětově bezpečný a zároveň nemá garbage collector.

Program se zkompiluje pomocí pomocného programu `cargo`, který je běžnou součástí jazyka. Funguje na stabilní verzi Rustu. Vybuildí se pomocí příkazu `cargo build --release` (to release je důležité kvůli optimalizacím, neoptimilizované programy jsou v rustu velmi pomalé).

Pro spuštění potřebuje argumenty, povinný je soubor se zadáním, volitelným argumentem je soubor pro kontrolu nalezených cen. Je v něm vestavěno měření času, ve kterém se počítá pouze čas pro rekurzivní navštěvovací funkci.

Je potřeba mu zadat, který engine má použít pro procházení prostoru. Aktuálně je k dispozici hrubá síla (pomocí argumentu `--naive`) a prořezávání (`--pruning`). Jde také vynutit kontrukční řešení pomocí `--force-construction`, ikdyž zadání říká, že by mělo být rozhodovací (opačně to nejde, protože chybí cílová cena).

Příklad zadání argumentů: `knapsack ../NR32_inst.dat --pruning`

Výstup programu
---------------

Na prvním řádku je `time`, to je čas na vyhodnocení jednoho problému, bez počítání doby na načtení a vypsání.

Na dalším řádku je vypsané řešení ve stejném formátu, v kterém je v zadání. Tedy postupně `id`, `size`, `cost` a `1/0` znázornující, jestli předmět do výsledké konfigurace patří a nebo nepatří. V případě nenalezení řešení v rozhodovacím problému je `cost` nula a bity znázurnující konfiguraci nejsou vypsány. V případě konstruktivní úlohy s porovnáváním, jestli našel správné řešení může být vypsána hláška `Same cost, but different solution!` - tedy výsledné naskládání předmětů bylo jiné, než v referenčním řešení, ale dalo to správnou cenu.

Na posledním řádku je Celkový čas, který je vypočítán jako součet všech `time` jednotlivých podúloh.

Algoritmy jsou `--naive`, `--pruning`, `--dynamic-weight`, `--dynamic-cost`, `--greedy` a `--redux`. Poslední řešič je `--ftpas`, který očekává číslo - vynucený dělitel, kterým se vydělí všechny ceny při výpočtu viz. v kapitole pro FTPAS.

```
time: 60.762µs
493 25 28680 1 0 1 1 1 0 1 1 1 0 1 1 1 0 1 1 0 1 1 0 0 0 1 1 1
time: 298ns
496 25 0
...
Maximum time: 1.420254ms Average time: 11.261µs
Total time: 5.630666ms
```

Naivní algoritmus
-----------------

Naivní algoritmus hloupě prohledává všechny konstrukce rekurzivním sestupem, pocestě si akorát průběžně počítá váhu a cenu.

Prořezávací algoritmus
----------------------

Prořezávací algoritmus používá také rekurzivní sestup, ale je v něm několik optimalizací.

Předměty se seřadí podle poměru `cost / weight` sestupně, takže se první zkouší do batohu dát ty předměty s lepším poměrem a zároveň u toho se vyfiltrují příliš těžké předměty. (optimalizace filtrace a řazení)

Také se bere v úvahu zbývajicí kapacita a když poměr `cost / weight` aktuálního předmětu vynásobená zbývajicí kapacitou nepřekoná nejlepší výsledek, tak se vrátím. Tady se předpokládá, že nejhorší scénář je, že všechny další předměty budou mít stejný `cost / weight` a tím approximuju maximální cenu, kterou můžou dát při zbývajicí kapacitě. (Optimalizace cost/weight ratia)

Počátek nainicializuju tak, že nazačátku nastavím největší item jako řešení a první navštívená konfigurace je konfigurace greedy řešení. Při $1.$ navštívené konfiguraci mám jistotu, že mám řešení alespoň $50%$ maxima možného. Tímpádem mám jistotu, že dokážu celkem efektivně ořezávat, když to je možné.

Dynamický programování - rozklad podle váh
------------------------------------------

Algoritmus je realizován tabulkou a obsahuje pouze optimilizaci filtrace a řazení. Tabulka má indexy `table[item][weight]`. Hodnota v tabulce má význam, že když vemu itemy $[item, len)$, tak dají cenu hodnoty v tabulce do dané váhy. Váha je optimalizovaná tak, že všechny váhy se vydělí nejvetším společným dělitelem všech vah. Někdy to může ušetřit pamět, pokud není nějaká váha prvočíslo. Je to algoritmus zpětné rekurze vyplnování pomocí DFS. Rekurze je implementovaná pomocí zásobníku.

Dynamický programování - rozklad podle cen
------------------------------------------

Algoritmus je realizován podobně jak v rozkladu podle váh. Tabulka má indexy `table[item][cost]` a hodnota určuje nejmenší možnou váhu, kterou dosáhnu přidáním předmětů $[0, item)$. Cena je opět vydělená podle největšího společného dělitele všech cen. Algoritmus je realizován BFS průchodem.

Opět algoritmus obsahuje optimalizaci filtrace a řazení, ale navíc v této verzi je realizované i ořezávání podle cost/weight ratia. Narozdíl od prořezávání to nepomůže násobně, ale jen o jednotky až desítky procent. Aby jsme mohli hned relativně efektivně prořezávat, tak první předpočítá řešení pomocí redux metody.

FTPAS
-----

Tento algoritmus je naprosto totožný, jak rozklad podle cen. Jediný rozdíl je, že před předáním do rozkladu podle cen se první všechny ceny vydělí vynuceným dělitelem a modulo ceny se zanedbá. Při zadání čísla, které je mocnina dvojky, tak metoda degraduje na metodu zanedbání bitů z přednášky.

Označíme si počet předmětů jako $n$, vynucený dělitel jako $gcd$. Naivně se dá maximální chyba odhadnout jako $n . (gcd - 1)$. Nicméně když máme už konkrétní instanci, tak to můžem udělat lépe. Můžeme spočítat, kolik nejvýše se vejde předmětů do batohu, označím $m$ (například seřadíme váhy vzestupně a berem tolik předmětů, kolik se jich vejde do batohu). Pak seřadíme zbytky cen po vydělení $gcd$ a součet prvních nejvyších $m$ zbytků je maximální možná chyba.

Greedy a Redux
--------------


Testovací stroj
---------------

Testování probíhá na procesoru `AMD 2700X` s frekvencí staticky nastavenou na $4,1GHz$. Systém je `ArchLinux` virtualizovaný pomocí Windows WSL. Podle testování to fungovalo rychleji, než nativně o víc jak $10\%$. (Např. rozklad podle ceny na instanci `NK40` trval nativně celkem $3,55s$ a ve WSL $2,53s$. Přepokládám, že to je spíš vyspělostí Rustu/LLVM na Linuxu oproti Windows spíš, než čímkoliv jiným.)

Testování exaktních metod
-------------------------

Pro testování výkonnosti jsem na všech 3 setech (NK, ZKC, ZKW) spustil pro všechny velikosti 10x všechny solvery, kromě naivního, který jsem spustil jen do velikosti 22. Celkem to trvalo 15 min, což není hrozný vzhledem k tomu, že se vše opakovalo 10x. Průměry i maxima jsem zprůměroval.

<span style="color:red">Červená</span> je naivní, <span style="color:purple">fialová</span> prořezávání, <span style="color:blue">modrá</span> rozklad podle váhy a <span style="color:green">zelená</span> rozklad podle ceny. <span style="text-decoration: underline">Plná čára je průměr</span> a <span style="text-decoration:underline dashed">přerušovaná maximum</span>. Na grafech je čas v logaritmickém měřítku, aby tam šlo něco rozpoznat u rychlejších metod.

![](NK.png)

![](ZKC.png)

![](ZKW.png)

Následuje tabulka s přesnýma číslama k velikosti instance 20 a 40.

|  | prořezávání | rozklad podle váh | rozklad podle cen |
|---------|--:|--:|--:|
| `NK20` |
| průměrný čas | $12,2µs$ | $144,0µs$ | $416,8µs$ |
| maximální čas | $123,5µs$ | $573,4µs$ | $23,4ms$ |
| `NK40` |
| průměrný čas | $23,3ms$ | $658,4µs$ | $5,1ms$ |
| maximální čas | $839,4ms$ | $2,7ms$ | $162,7ms$ |
||
| `ZKC20` |
| průměrný čas | $17,7µs$ | $159,9µs$ | $1,4ms$ |
| maximální čas | $65,2µs$ | $392,1µs$ | $83,5ms$ |
| `ZKC40` |
| průměrný čas | $10,8ms$ | $723,5µs$ | $7,7ms$ |
| maximální čas | $157,1ms$ | $2,0ms$ | $159,9ms$ |
||
| `ZKW20` |
| průměrný čas | $1,7µs$ | $9,3µs$ | $620,2µs$ |
| maximální čas | $7,0µs$ | $83,5µs$ | $9,4ms$ |
| `ZKW40` |
| průměrný čas | $2,5µs$ | $15,9µs$ | $1,6ms$ |
| maximální čas | $9,6µs$ | $204,6µs$ | $10,2ms$ |

Naivní funguje na všech třech instancích stejně, což je očekáváné. Zajimavé je, že všechny ostatní metody fungují na sadě profesora zlomyslného rychleji než na normální. U ZKW to je tím, že je tam spousta předmětů, která je přes maximální kapacitu a tak se ani do samotných algoritmů nedostanou, ale ořeže je předzpracování.

Podle předpokladu Naivní a prořezávání se chová exponencionálně. Kromě zmíněného ZKW, kde se tam velká část předmětů nedostane a tak čas má pořád na úrovni malé instance, kde je zároveň rychlejší jak rozklady.

Rozklad podle ceny je pomalejší než rozklad podle váhy, ale jeho význam je použití ve FTPAS. Oba rozklady se chovají lineárně.

Obecně platí, že pro malé instance je nejlepší prořezávání, ale jakmile je velikost dostatečně velká, tak začíná být lepší rozklad podle váhy a dá se předpokládat, že ten se tam udrží

Testování approximačních metod
------------------------------

Testování probíhá na instanci `NK40`, která má 500 problémů. Greedy ani neuvádím, na sadě `NK40` dopadlo stejně, jak redux a je rychlejší o pár desetin mikrosekundy, které se dají spíš přisuzovat náhodě. Na sadě `NK37` mělo greedy o jednu chybu víc.

Průměrná a relativní chyba je spočítaná jen přes chybující problémy, správné do výpočtu nejsou započítaný. Výpočet maximální chyby je nastíněný v části FTPAS.

| `NK40`                    | redux   | ftpas 1   | ftpas 2     | ftpas 5   | ftpas 10  | ftpas 50 | ftpas 100 | ftpas 200 |
|---------------------------|--------:|----------:|------------:|----------:|----------:|---------:|----------:|----------:|
| průměrný čas              | $3,3µs$ | $5,0ms$   | $2,0ms$     | $782,1µs$ | $427,4µs$ | $92,4µs$ | $51,9µs$  | $27,2µs$  |
| maximální čas             | $8,5µs$ | $155,0ms$ | $76,8ms$    | $30,1ms$  | $15,3ms$  | $2,8ms$  | $1,5ms$   | $492,8µs$ |
| počet chyb                | $260$   | $0$       | $1$         | $9$       | $12$      | $60$     | $121$     | $213$     |
| průměrná chyba            | $218,9$ | $0$       | $2$         | $2,3$     | $3,1$     | $27,2$   | $53,5$    | $111,7$   |
| průměrná relativní chyba  | -       | -         | $12,5\%$    | $3,3\%$   | $2,1\%$   | $3,4\%$  | $3,4\%$   | $3,6\%$   |
| maximální relativní chyba | -       | -         | $12,5\%$    | $4,8\%$   | $6,3\%$   | $14,3\%$ | $16,9\%$  | $12,7\%$  |

| `ZKC40`                   | redux   | ftpas 1   | ftpas 2     | ftpas 5   | ftpas 10  | ftpas 50  | ftpas 100 | ftpas 200 |
|---------------------------|--------:|----------:|------------:|----------:|----------:|----------:|----------:|----------:|
| průměrný čas              | $3,2µs$ | $7,59ms$  | $2,0ms$     | $1,1ms$   | $544,5µs$ | $122,8µs$ | $61,6µs$  | $31,1µs$  |
| maximální čas             | $8,54s$ | $157,5ms$ | $76,8ms$    | $32,1ms$  | $15,6ms$  | $2,8ms$   | $1,4ms$   | $539,9µs$ |
| počet chyb                | $485$   | $0$       | $71$        | $229$     | $327$     | $453$     | $474$     | $500$     |
| průměrná chyba            | $884,9$ | $0$       | $1,3$       | $3,4$     | $6,2$     | $28,2$    | $66,5$    | $415,2$   |
| průměrná relativní chyba  | -       | -         | $6,5\%$     | $4,4\%$   | $3,6\%$   | $3,1\%$   | $3.5\%$   | $11,2\%$  |
| maximální relativní chyba | -       | -         | $16,6\%$    | $14,1\%$  | $13,1\%$  | $8,1\%$   | $21,2\%$  | $32,7\%$  |

Dataset `NKW` jsem vynechal, protože většina předmětů je vyfiltrována a tak tam moc správných odpovědí ani nezůstává. Např. `NKW40` s dělitelem 256 obsahoval pouze 3 chyby.

FTPAS časově dopadl podle očekávání, při gcd 1 dopadl nastejno s rozkladem podle cen, a pak lineárně zvyšoval čas podle zvoleného gcd. Redux je velmi jednoduchá taktika jen s jedním řazením, tak překonat ho složitějším FTPASem je možný až když jeho přesnost začíná být zoufalá.

Ohledně chyb na `NK40` dopadl silně nad očekávání. dokonce i s největším dělitelem 200 chyboval v méně než polovině instancí. Dokonce tam kde chyboval, tak procento možný chyby je pouze 4%.



Závěr
-----

V exaktních metodách se mě povedlo výrazně vylepšit ořezávání, že dokonce největší instance je velice rychlá. Překvapením pro mě bylo zjištění, že `NK40` je hůře ořezatelná, než `ZKC`, která by měla podle instrukcí být navržená tak, aby nebyla ořezatelná. Provedl jsem oba rozklady a mám z toho závěr, že rozklad podle váhy je vhodnější, pravděpodobně protože je možné lépe omezit maximální váhu, než cenu. Rozklad podle váhy má výhodu v approximaci s povolenou chybou, protože je možné určit, kolik je její hodnota.

Rozklady mají mnohem větší overhead při menších instancích. Zvlášt rozklad podle ceny, na to je citlivý. Pro malé instance se tedy víc vyplatí prořezávání. Od určité velikosti ale vyhrává dynamické programování, pokud jsou ceny a váhy v rozumném rozpětí.

Metoda FTPAS dopadla výborně na datasetu `NK`, kde při zvoleném děliteli 10 překonala rychlost rozkladu podle váhy s minimálním počtem chyb. Výrazně hůř dopadla na datasetu `ZKC`, kdy při překonání rychlosti rozkladu podle váhy už chybovala ve více než polovině instancí. Všechny problémy v datasetu `NKC40` mají cílovou cenu přes $30 000$, tedy průměrná chyba $6,2$ není až tak významná.
