verifica che dopo questo refactoring abbiamo lasciato la situazione pulita, cioè non abbiamo fatto minimo effort per far compilare o green test ma piuttosto il codice deve essere pulito. no rimasugli da design vecchio, no reexport solo per build ecc. deve essere un
progetto pulito, ben manutenuto. verifica che non ci siano altri file enormi penso 500LOC è accettabile ma sopra no a meno che non strettamente necessario. separa sorgente da tests se necessario. verifica best practices per ai coding agent riguardo dimensione files.
verifica che i test siano veramente utili e che abbiano un reale scopo e che non stiano li solo perchè vecchio design, old tests, useless tests. test che non falliscono mai = inutili
verifica di non aver cancellato codice effettivamente utile che è realmente nuovo design creato da m021 e dal refactoring
nel fare refactoring prio è situazione pulita non minimo effort per raggiungere build e green test
