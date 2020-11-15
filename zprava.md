<link rel="stylesheet" type="text/css" href="pandoc.css"/>

Problém batohu
==============

Definice problému
-----------------

Je daná množina předmětů s váhou a cenou. Úkolem je najít takovou podmnožinu, aby součet váh byl do maximální povolené váhy a zároveň celková cena podmnožiny byla maximální (konstrukční problém).

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

Barvy v grafu
-------------

<span style="color:red">Červená</span> je naivní, <span style="color:purple">fialová</span> prořezávání, <span style="color:blue">modrá</span> rozklad podle váhy a <span style="color:green">zelená</span> rozklad podle ceny. <span style="text-decoration: underline">Plná čára je průměr</span> a <span style="text-decoration:underline dotted">přerušovaná maximum</span>. Na grafech je čas v logaritmickém měřítku.

Vylepšení
---------

Prořezávání prořezává mnohem lépe, proto sadu NK řeší mnohem rychleji, než implementace v 2. iteraci. Nově se bere v úvahu navíc zbývajicí cena a spočítá se přesně maximální dosáhnutelná cena v $O(ln n)$ času. To je možné tím, že mám pole součtu vah od určitého indexu do posledního, stejně s cenama. Poté binárně vyhledávám váhu takovou, že se akorát vejde zbývajicí kapacita. Vzhledem k tomu, že předměty jsou seřazený podle `cost/weight` ratia, tak když vemu předchozí předměty a poměrovou část poslední, tak to je maximální dosažitelná cena. Přikládám graf vylepšeného prořezávání na sadě NK. Během minulé iterace se prořezávání protínalo s rozkladem podle ceny. Sady `ZKC` a `ZKW` se téměř nezměnili.

![](NK.png)

Předměty se nově neřadí pouze podle `cost/weight` ratia, ale navíc se řadí od nejtěžšího po nejlehčí, pokud mají stejný poměr. To zaručí, že stejné předměty jsou vedle sebe a díky tomu je možné filtrovat permutace stejného předmětu v prořezávání (stejným předmětem se myslí předmět, co má stejnou cenu i váhu s jiným).

Jednotlivé algoritmy jsou nově rozdělené do jednotlivých souborů.

Naivní implementace
-------------------

Naivní implementace je necitlivá na jakýkoliv kombinace cen, váh. Ta projde naivně všechny kombinaci a tedy se chová čistě exponencionálně - nemá smysl měřit vliv jednotlivých vstupů, protože jediné na co reaguje je velikost.

Robustnost
----------

Všechny zbývajicí metody přeuspořádají předměty podle `cost/weight` ratia primárně a podle váhy sekundárně (oboje sestupně). Vzhledem k tomu, že si předměty deterministicky seřadí před samotným spuštěním algoritmu, tak nemá smysl měřit vliv permutací, protože jediné co ovlivní je maximálně samotné řazení. V předchozí zprávě vycházel čas Reduxu maximálně $8,5µs$ (velikost instance 40). V této metodě je hlavní brzda právě samotné řazení - dá se očekávat, že řazení před spuštěním algoritmu se podílí na času při velikosti instance 40 přibližně takovýmhle časem. Všechny metody jsou tímto robustní a je zbytečné to experimentálně ověřovat.

Prvotní testování parametrů
---------------------------

Na začátek zkusím vygenerovat instance (500) s výchozíma parametrama, maximální cenu i váhu nastavím na $1 000$, poměr kapacity/součtu $0,8$ a velikosti jako byli v dodaných instancích, abych to uměl porovnat.

![](PP.png)

Podle očekávání tato sada dopadla srovnatelně s NK. Pojmenuji ji jako PP (první pokus).

Zvyšování ceny a váhy
---------------------

Provedu 2 pokusy, první zvýšim jen maximální cenu na $10\times 1 000$, poté jen váhu. Předpoklad je, že takováta modifikace by měla vadit jen rozkladu podle ceny a váhy podle toho, co zvyšuji. Sada MC je s nastavením jako PP, akorát cena je na $10 000$. Sada MW obdobně pro váhu.

![](MC.png)

![](MW.png)

Dopadlo to podle očekávání - zvyšování ceny zhoršilo pouze čas na rozkladu podle ceny a zvyšování váhy pouze čas na rozkladu podle váhy.

Korelace váhy a ceny
--------------------

Tady udělám 2 sady: CC a SC. CC je shodná s nastavením PP, ale korelace je na `corr`. SC má korelaci na `strong`. Předpoklad je, že nejvíce to bude vadit prořezávání a rozkladům by to nemělo vadit a nebo lehce pomoct.

![](CC.png)

![](SC.png)

Přesně podle předpokladu větší korelace zpomaluje prořezávání. Navíc trochu zpomaluje i rozklad podle ceny, protože tam mám taky ořezávání. Ikdyž je tam menší než v samotném prořezávání.

Poměr kapacity k sumární váze
-----------------------------

V této sadě zafixuji velikost na 40 a budu hýbat s poměrem kapacity. Ostatní nastavení bude podle PP. Předpoklad je, že nějaký poměr bude nejtěžší a bude to okolo něho tvořit přibližně tvar normální rozdělení. Rozklady budou mít nejmenší hodnotu relativně vysoko díky větší konstantní složitosti a naopak prořezávání půjde k hodně malým číslům, jak bude moct mnohem lépe prořezávat. Navíc i rozklady by měli růst, ale jestli budou mít maxima nedokážu odhadnout.


![](BR.png)

Při nekorelované váze a ceně je tvar jiný, než jsem ho naměřil při pokusném spouštění. Proto zkusím ještě udělat datové soubory CR s SR (korelace na `corr` a `strong`).

![](CR.png)

![](SR.png)

Při nekorelované váze a ceně je nejdelší prořezávání asi okolo poměru $0,55$. Při korelované verzi je vrchol lehce nad $0,8$. Prořezávání je hodně citlivé na tento poměr a při vhodné korelaci se výpočet prodlouží.

Oba rozklady rostou přibližně lineárně s poměrem - což je efekt toho, že se zvyšuje lineárně maximální kapacita a maximální cena, zatímco předměty mají pořád stejný rozsah cen a vah.

Vliv nepoměru věcí
------------------

Předpokládám, že to nejvíc bude ovlivnovat prořezávání - tedy rovnou zafixuji korelaci na `corr` a `strong`. $-1$ v grafu odpovídá nastavení `light` a $k = 1$. $1$ v grafu odpovídá nastavení `heavy` a $k = 1$. 0 odpovídá nastavení `bal`. `CB` je typicky s korelací na `corr` a `SB` s korelací `strong`.

![](CB.png)

![](SB.png)

Vliv nepoměru věcí má spíše malý vliv. Rozklady se zvyšovali s tím, jak se zvyšovala maximální kapacita a cena. U prořezávání to záleželo na míře korelace, ale rozdíly mezi vyváženíma byli cca do $2\times$.


Závěr
-----

Rozklad podle ceny je citlivý poze na maximální možnou cenu a velikost instance. Rozklad podle váhy stejně akorát na váhu.

V případě, že předměty nemají korelovanou váhu a cenu, tak bude vycházet nejlíp prořezávání - instance velikosti 40 zdaleka není limit.

V případě korelované váhy je nejtěžší obsazenost $0,8$ kapacity. V případě nekorelované se to posune na $0,55$.

Nevyváženost má na celkový čas spíše malý vliv, u rozkladů jde vidět lehce lineární, ale ona zároveň stoupá maximální cena a kapacita batohu, když je více těžších předmětů. Prořezávání víc vyhovuje převaha těžších předmětů v případě silné korelace. Naopak lehce lehčí uvítá v případě slabší korelaci.
