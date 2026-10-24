import { CommonModule } from '@angular/common';
import { Component, inject } from '@angular/core';
import { MatButtonModule } from '@angular/material/button';
import { MAT_DIALOG_DATA, MatDialogModule, MatDialogRef } from '@angular/material/dialog';
import { MatIconModule } from '@angular/material/icon';

import { Tenant } from '../domain/tenant';

export interface ConfirmacaoProducaoDados {
  tenant: Tenant;
}

/** Modal de confirmação para migrar um escritório de TRIAL para PRODUÇÃO
 * (SPEC §8): usa credenciais reais, avança a versão e pode ser bloqueado
 * pelo backend se faltar credencial ou certificado A1 válido. */
@Component({
  selector: 'app-confirma-producao-dialog',
  standalone: true,
  imports: [CommonModule, MatDialogModule, MatButtonModule, MatIconModule],
  template: `
    <h2 mat-dialog-title>
      <mat-icon class="aviso-icono" aria-hidden="true">warning</mat-icon>
      Migrar {{ dados.tenant.nome }} para produção?
    </h2>
    <mat-dialog-content>
      <p>
        O escritório <strong>{{ dados.tenant.slug }}</strong> usará as credenciais de
        produção do SERPRO. A versão do escritório avança e o consumo passa a valer
        regras reais.
      </p>
      <p class="aviso" role="alert">
        A migração é bloqueada quando falta a credencial de produção
        (INTEGRA_CONTADOR) ou o certificado A1 em validade.
      </p>
    </mat-dialog-content>
    <mat-dialog-actions align="end">
      <button mat-button (click)="cancelar()">Voltar ao trial</button>
      <button mat-raised-button color="warn" (click)="confirmar()">Migrar para produção</button>
    </mat-dialog-actions>
  `,
  styles: `
    .aviso {
      color: #b71c1c;
    }

    .aviso-icono {
      vertical-align: middle;
      color: #e65100;
      margin-right: 8px;
    }
  `,
})
export class ConfirmaProducaoDialog {
  private readonly dialogoRef = inject<MatDialogRef<ConfirmaProducaoDialog, boolean>>(MatDialogRef);
  protected readonly dados = inject<ConfirmacaoProducaoDados>(MAT_DIALOG_DATA);

  protected confirmar(): void {
    this.dialogoRef.close(true);
  }

  protected cancelar(): void {
    this.dialogoRef.close(false);
  }
}
