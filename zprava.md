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

Na prvním řádku je `time`, to je čas na vyhodnocení jednoho problému, bez počítání doby na načtení a vypsání. Dále tam je `visits`, což je návštěva koncových konfigurací. Poslední na prvním řádku je `p_visits`, což je počet navštívených nekoncových vrcholů.

Na dalším řádku je vypsané řešení ve stejném formátu, v kterém je v zadání. Tedy postupně `id`, `size`, `cost` a `1/0` znázornující, jestli předmět do výsledké konfigurace patří a nebo nepatří. V případě nenalezení řešení v rozhodovacím problému je `cost` nula a bity znázurnující konfiguraci nejsou vypsány. V případě konstruktivní úlohy s porovnáváním, jestli našel správné řešení může být vypsána hláška `Same cost, but different solution!` - tedy výsledné naskládání předmětů bylo jiné, než v referenčním řešení, ale dalo to správnou cenu.

Na posledním řádku je Celkový čas, který je vypočítán jako součet všech `time` jednotlivých podúloh.

```
time: 60.762µs visits: 93 p_visits: 9332
493 25 28680 1 0 1 1 1 0 1 1 1 0 1 1 1 0 1 1 0 1 1 0 0 0 1 1 1
time: 298ns visits: 0 p_visits: 1
496 25 0
...
Total time: 110.946808ms
```

Naivní algoritmus
-----------------

První jsem implementoval naivní algoritmus, ten zkouší každou iteraci rekurzivním průchodem. Po cestě si nasčítává cenu a váhu přidaných itemů, nicméně až nakonci se rozhoduje, jestli překročil váhu, a jestli má maximální cenu.

Prořezávací algoritmus
----------------------

Funkčnost tohoto algoritmu je podobná jak naivní až na dvě optimalizace. Průběžně zkouší, jestli překročil váhu a při překročení nepokračuje. Navíc kontroluje jestli ve zbývajicích předmětech je dostatečná hodnota na překročení minimální akceptovatelné ceny (u konstruktivní verze to je aktuálně maximální nalezené cena), a když není tak se také ukončí. Vzhledem k optimalizacím `visits` u rozhodovací verze je 1 nebo 0, podle toho jestli řešení našel. U kontruktivní verze to je počet vylepšení aktuálního řešení.

Porovnání implementací
----------------------

Testování provádím na notebooku s procesorem `i5-8350U` a dostatkem RAM paměti. Při měření dbám akorát na to, aby notebook byl pokaždé zapojen do síťě a teda se nesnažil šetřit energii. Vzhledem k tomu, že u rozhodovací verze v prořezávacím algoritmu se navštíví jediná konfigurace, a to je ta, která má cenu alespoň jako je minimální povolená cena, tak budu porovnávat parametry `p_visists`, což je počet návštěv nekoncových vrcholů v konfiguraci (tj. tam kde se začne rozhodovat, jestli tam ten předmět dá, nebo nedá).

Už z prvotního spouštění jde poznat, že rozdíly implementovaných algoritmů jsou velké. Například testování celého souboru `NR25_inst.dat` ze zadání trvá naivní implementaci celkově 53 sekund a prořezávacímu algoritmu 22 až 42 ms ().
