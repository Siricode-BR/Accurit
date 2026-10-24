import { CommonModule } from '@angular/common';
import { Component, inject, OnInit, signal } from '@angular/core';
import { FormBuilder, FormGroup, ReactiveFormsModule, Validators } from '@angular/forms';
import { Router, ActivatedRoute } from '@angular/router';
import { MatCardModule } from '@angular/material/card';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';
import { MatSelectModule } from '@angular/material/select';
import { MatProgressSpinnerModule } from '@angular/material/progress-spinner';
import { MatSnackBarModule, MatSnackBar } from '@angular/material/snack-bar';

import { ClienteService, CreateClienteRequest, UpdateClienteRequest, TipoCliente, PorteRegime } from './cliente.service';

@Component({
  selector: 'app-cliente-form',
  standalone: true,
  imports: [
    CommonModule,
    ReactiveFormsModule,
    MatCardModule,
    MatFormFieldModule,
    MatInputModule,
    MatButtonModule,
    MatIconModule,
    MatSelectModule,
    MatProgressSpinnerModule,
    MatSnackBarModule,
  ],
  template: `
    <mat-card>
      <mat-card-header>
        <mat-card-title>{{ isEditing ? 'Editar Cliente' : 'Novo Cliente' }}</mat-card-title>
        <mat-card-subtitle>{{ isEditing ? 'Atualize os dados do cliente' : 'Cadastre um novo cliente no escritório' }}</mat-card-subtitle>
      </mat-card-header>
      <mat-card-content>
        <form [formGroup]="form" (ngSubmit)="onSubmit()" class="cliente-form">
          <div class="form-section">
            <h3>Tipo e Classificação</h3>
            <div class="form-row">
              <mat-form-field appearance="outline" class="full-width">
                <mat-label>Tipo</mat-label>
                <mat-select formControlName="tipo">
                  <mat-option value="PF">Pessoa Física (CPF)</mat-option>
                  <mat-option value="PJ">Pessoa Jurídica (CNPJ)</mat-option>
                </mat-select>
                @if (form.get('tipo')?.hasError('required')) {
                  <mat-error>Tipo é obrigatório</mat-error>
                }
              </mat-form-field>
            </div>
            <div class="form-row">
              <mat-form-field appearance="outline" class="full-width">
                <mat-label>Porte/Regime</mat-label>
                <mat-select formControlName="porteRegime">
                  <mat-option value="">Selecione (obrigatório para PJ)</mat-option>
                  <mat-option value="MEI">MEI</mat-option>
                  <mat-option value="SIMPLES_NACIONAL">Simples Nacional</mat-option>
                  <mat-option value="LUCRO_PRESUMIDO">Lucro Presumido</mat-option>
                  <mat-option value="LUCRO_REAL">Lucro Real</mat-option>
                </mat-select>
              </mat-form-field>
            </div>
          </div>

          <div class="form-section">
            <h3>Identificação</h3>
            <div class="form-row">
              <mat-form-field appearance="outline" class="full-width">
                <mat-label>{{ form.value.tipo === 'PJ' ? 'CNPJ' : 'CPF' }}</mat-label>
                <input matInput formControlName="cpfCnpj" [placeholder]="form.value.tipo === 'PJ' ? '12.345.678/0001-90' : '123.456.789-00'" />
                @if (form.get('cpfCnpj')?.hasError('required')) {
                  <mat-error>{{ form.value.tipo === 'PJ' ? 'CNPJ' : 'CPF' }} é obrigatório</mat-error>
                }
                @if (form.get('cpfCnpj')?.hasError('cpfInvalido')) {
                  <mat-error>CPF inválido</mat-error>
                }
                @if (form.get('cpfCnpj')?.hasError('cnpjInvalido')) {
                  <mat-error>CNPJ inválido</mat-error>
                }
                @if (form.get('cpfCnpj')?.hasError('tipoIncompativel')) {
                  <mat-error>Documento não corresponde ao tipo selecionado</mat-error>
                }
              </mat-form-field>
            </div>
          </div>

          <div class="form-section">
            <h3>Dados Cadastrais</h3>
            <div class="form-row">
              <mat-form-field appearance="outline" class="full-width">
                <mat-label>Nome / Razão Social</mat-label>
                <input matInput formControlName="nomeRazaoSocial" placeholder="Nome completo ou razão social" maxlength="200" />
                @if (form.get('nomeRazaoSocial')?.hasError('required')) {
                  <mat-error>Nome / Razão Social é obrigatório</mat-error>
                }
                @if (form.get('nomeRazaoSocial')?.hasError('maxlength')) {
                  <mat-error>Nome deve ter no máximo 200 caracteres</mat-error>
                }
              </mat-form-field>
            </div>
            <div class="form-row">
              <mat-form-field appearance="outline" class="full-width">
                <mat-label>Nome Fantasia</mat-label>
                <input matInput formControlName="nomeFantasia" placeholder="Nome fantasia (opcional)" maxlength="200" />
                @if (form.get('nomeFantasia')?.hasError('maxlength')) {
                  <mat-error>Nome fantasia deve ter no máximo 200 caracteres</mat-error>
                }
              </mat-form-field>
            </div>
            <div class="form-row">
              <mat-form-field appearance="outline" class="full-width">
                <mat-label>E-mail</mat-label>
                <input matInput type="email" formControlName="email" placeholder="contato@exemplo.com" />
                @if (form.get('email')?.hasError('email')) {
                  <mat-error>E-mail inválido</mat-error>
                }
              </mat-form-field>
            </div>
            <div class="form-row">
              <mat-form-field appearance="outline" class="full-width">
                <mat-label>Telefone</mat-label>
                <input matInput formControlName="telefone" placeholder="(11) 99999-9999" />
              </mat-form-field>
            </div>
          </div>

          <div class="form-actions">
            <button mat-button type="button" routerLink="/clientes">
              <mat-icon>arrow_back</mat-icon>
              Cancelar
            </button>
            <button mat-raised-button color="primary" type="submit" [disabled]="form.invalid || enviando()">
              @if (enviando()) {
                <mat-spinner diameter="20"></mat-spinner>
                <span>Salvando...</span>
              } @else {
                <ng-container>
                  <mat-icon>{{ isEditing ? 'save' : 'person_add' }}</mat-icon>
                  <span>{{ isEditing ? 'Atualizar' : 'Cadastrar' }}</span>
                </ng-container>
              }
            </button>
          </div>
        </form>
      </mat-card-content>
    </mat-card>
  `,
  styles: `
    .cliente-form {
      display: flex;
      flex-direction: column;
      gap: 24px;
    }

    .form-section {
      display: flex;
      flex-direction: column;
      gap: 16px;

      h3 {
        margin: 0 0 16px;
        font-size: 1.1rem;
        font-weight: 500;
        color: #333;
      }
    }

    .form-row {
      display: flex;
      gap: 16px;
      flex-wrap: wrap;

      .full-width {
        flex: 1;
        min-width: 280px;
      }
    }

    .form-actions {
      display: flex;
      justify-content: flex-end;
      gap: 16px;
      padding-top: 8px;
      border-top: 1px solid #e0e0e0;

      button {
        display: flex;
        align-items: center;
        gap: 8px;
      }

      mat-spinner {
        margin-right: 8px;
      }
    }
  `,
})
export class ClienteFormComponent implements OnInit {
  private readonly fb = inject(FormBuilder);
  private readonly clienteService = inject(ClienteService);
  private readonly router = inject(Router);
  private readonly route = inject(ActivatedRoute);
  private readonly snackBar = inject(MatSnackBar);

  protected readonly enviando = signal(false);

  protected readonly form: FormGroup = this.fb.group({
    tipo: ['PF', Validators.required],
    porteRegime: [''],
    cpfCnpj: ['', Validators.required],
    nomeRazaoSocial: ['', [Validators.required, Validators.maxLength(200)]],
    nomeFantasia: ['', Validators.maxLength(200)],
    email: ['', Validators.email],
    telefone: [''],
  });

  protected readonly porteRegimeOptions: { value: PorteRegime; label: string }[] = [
    { value: 'MEI', label: 'MEI' },
    { value: 'SIMPLES_NACIONAL', label: 'Simples Nacional' },
    { value: 'LUCRO_PRESUMIDO', label: 'Lucro Presumido' },
    { value: 'LUCRO_REAL', label: 'Lucro Real' },
  ];

  ngOnInit(): void {
    const id = this.route.snapshot.paramMap.get('id');
    if (id) {
      this.carregarCliente(id);
    }

    // Watch for tipo changes to update validations
    this.form.get('tipo')?.valueChanges.subscribe((tipo) => {
      this.atualizarValidacoesCpfCnpj(tipo);
    });
  }

  private atualizarValidacoesCpfCnpj(tipo: TipoCliente): void {
    const cpfCnpjControl = this.form.get('cpfCnpj');
    const porteRegimeControl = this.form.get('porteRegime');

    // Clear existing validators
    cpfCnpjControl?.clearValidators();
    cpfCnpjControl?.setValidators([Validators.required]);

    if (tipo === 'PJ') {
      porteRegimeControl?.setValidators([Validators.required]);
    } else {
      porteRegimeControl?.clearValidators();
      porteRegimeControl?.setValue('');
    }

    cpfCnpjControl?.updateValueAndValidity();
    porteRegimeControl?.updateValueAndValidity();
  }

  private carregarCliente(id: string): void {
    this.clienteService.obter(id).subscribe({
      next: (cliente) => {
        this.form.patchValue({
          tipo: cliente.tipo,
          porteRegime: cliente.porteRegime || '',
          cpfCnpj: cliente.cpfCnpj,
          nomeRazaoSocial: cliente.nomeRazaoSocial,
          nomeFantasia: cliente.nomeFantasia || '',
          email: cliente.email || '',
          telefone: cliente.telefone || '',
        });
        this.atualizarValidacoesCpfCnpj(cliente.tipo);
      },
      error: () => {
        this.snackBar.open('Erro ao carregar cliente', 'Fechar', { duration: 3000 });
        this.router.navigate(['/clientes']);
      },
    });
  }

  get isEditing(): boolean {
    return !!this.route.snapshot.paramMap.get('id');
  }

  async onSubmit(): Promise<void> {
    if (this.form.invalid) {
      this.form.markAllAsTouched();
      return;
    }

    this.enviando.set(true);

    const formValue = this.form.value;
    const cpfCnpjLimpo = formValue.cpfCnpj.replace(/\D/g, '');

    const request: CreateClienteRequest | UpdateClienteRequest = {
      tipo: formValue.tipo,
      porteRegime: formValue.porteRegime || undefined,
      cpfCnpj: cpfCnpjLimpo,
      nomeRazaoSocial: formValue.nomeRazaoSocial,
      nomeFantasia: formValue.nomeFantasia || undefined,
      email: formValue.email || undefined,
      telefone: formValue.telefone || undefined,
    };

    const operacao = this.isEditing
      ? this.clienteService.atualizar(this.route.snapshot.paramMap.get('id')!, request as UpdateClienteRequest)
      : this.clienteService.criar(request as CreateClienteRequest);

    operacao.subscribe({
      next: () => {
        this.snackBar.open(
          this.isEditing ? 'Cliente atualizado com sucesso!' : 'Cliente cadastrado com sucesso!',
          'Fechar',
          { duration: 3000 }
        );
        this.router.navigate(['/clientes']);
      },
      error: (error) => {
        const message = error.error?.error?.message || 'Erro ao salvar cliente';
        this.snackBar.open(message, 'Fechar', { duration: 5000 });
      },
      complete: () => {
        this.enviando.set(false);
      },
    });
  }
}