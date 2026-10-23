import { CommonModule } from '@angular/common';
import { Component, inject, signal } from '@angular/core';
import { FormBuilder, FormGroup, ReactiveFormsModule, Validators } from '@angular/forms';
import { Router, ActivatedRoute } from '@angular/router';
import { MatCardModule } from '@angular/material/card';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';
import { MatProgressSpinnerModule } from '@angular/material/progress-spinner';
import { MatSnackBarModule, MatSnackBar } from '@angular/material/snack-bar';

import { TenantService, CreateTenantRequest } from './tenant.service';

@Component({
  selector: 'app-tenant-create',
  standalone: true,
  imports: [
    CommonModule,
    ReactiveFormsModule,
    MatCardModule,
    MatFormFieldModule,
    MatInputModule,
    MatButtonModule,
    MatIconModule,
    MatProgressSpinnerModule,
    MatSnackBarModule,
  ],
  template: `
    <mat-card>
      <mat-card-header>
        <mat-card-title>Novo Escritório</mat-card-title>
        <mat-card-subtitle>Cadastre um novo escritório e seu titular</mat-card-subtitle>
      </mat-card-header>
      <mat-card-content>
        <form [formGroup]="form" (ngSubmit)="onSubmit()" class="tenant-form">
          <div class="form-section">
            <h3>Dados do Escritório</h3>
            <div class="form-row">
              <mat-form-field appearance="outline" class="full-width">
                <mat-label>Slug</mat-label>
                <input matInput formControlName="slug" placeholder="ex: meu-escritorio" maxlength="40" />
                <mat-hint>Somente letras minúsculas, números e hífen (3 a 40 caracteres)</mat-hint>
                @if (form.get('slug')?.hasError('required')) {
                  <mat-error>Slug é obrigatório</mat-error>
                }
                @if (form.get('slug')?.hasError('minlength') || form.get('slug')?.hasError('maxlength')) {
                  <mat-error>Slug deve ter entre 3 e 40 caracteres</mat-error>
                }
                @if (form.get('slug')?.hasError('pattern')) {
                  <mat-error>Slug deve conter apenas letras minúsculas, números e hífen</mat-error>
                }
              </mat-form-field>
            </div>
            <div class="form-row">
              <mat-form-field appearance="outline" class="full-width">
                <mat-label>Nome</mat-label>
                <input matInput formControlName="nome" placeholder="Razão social do escritório" maxlength="200" />
                @if (form.get('nome')?.hasError('required')) {
                  <mat-error>Nome é obrigatório</mat-error>
                }
                @if (form.get('nome')?.hasError('maxlength')) {
                  <mat-error>Nome deve ter no máximo 200 caracteres</mat-error>
                }
              </mat-form-field>
            </div>
          </div>

          <div class="form-section">
            <h3>Titular do Escritório</h3>
            <div class="form-row">
              <mat-form-field appearance="outline" class="full-width">
                <mat-label>E-mail</mat-label>
                <input matInput type="email" formControlName="emailTitular" placeholder="titular@exemplo.com" />
                @if (form.get('emailTitular')?.hasError('required')) {
                  <mat-error>E-mail é obrigatório</mat-error>
                }
                @if (form.get('emailTitular')?.hasError('email')) {
                  <mat-error>E-mail inválido</mat-error>
                }
              </mat-form-field>
            </div>
            <div class="form-row">
              <mat-form-field appearance="outline" class="full-width">
                <mat-label>CPF (somente números)</mat-label>
                <input matInput formControlName="cpfTitular" placeholder="12345678901" maxlength="11" />
                @if (form.get('cpfTitular')?.hasError('required')) {
                  <mat-error>CPF é obrigatório</mat-error>
                }
                @if (form.get('cpfTitular')?.hasError('minlength') || form.get('cpfTitular')?.hasError('maxlength')) {
                  <mat-error>CPF deve ter exatamente 11 dígitos</mat-error>
                }
                @if (form.get('cpfTitular')?.hasError('pattern')) {
                  <mat-error>CPF deve conter apenas números</mat-error>
                }
              </mat-form-field>
            </div>
            <div class="form-row">
              <mat-form-field appearance="outline" class="full-width">
                <mat-label>Senha</mat-label>
                <input matInput type="password" formControlName="senhaTitular" placeholder="Mínimo 8 caracteres" />
                @if (form.get('senhaTitular')?.hasError('required')) {
                  <mat-error>Senha é obrigatória</mat-error>
                }
                @if (form.get('senhaTitular')?.hasError('minlength')) {
                  <mat-error>Senha deve ter no mínimo 8 caracteres</mat-error>
                }
              </mat-form-field>
            </div>
          </div>

          <div class="form-actions">
            <button mat-button type="button" routerLink="/admin/tenants">
              <mat-icon>arrow_back</mat-icon>
              Voltar
            </button>
            <button mat-raised-button color="primary" type="submit" [disabled]="form.invalid || enviando()">
              @if (enviando()) {
                <mat-spinner diameter="20"></mat-spinner>
                <span>Cadastrando...</span>
              } @else {
                <ng-container>
                  <mat-icon>save</mat-icon>
                  <span>Cadastrar Escritório</span>
                </ng-container>
              }
            </button>
          </div>
        </form>
      </mat-card-content>
    </mat-card>
  `,
  styles: `
    .tenant-form {
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
export class TenantCreateComponent {
  private readonly fb = inject(FormBuilder);
  private readonly tenantService = inject(TenantService);
  private readonly router = inject(Router);
  private readonly snackBar = inject(MatSnackBar);

  protected readonly enviando = signal(false);

  protected readonly form: FormGroup = this.fb.group({
    slug: ['', [
      Validators.required,
      Validators.minLength(3),
      Validators.maxLength(40),
      Validators.pattern(/^[a-z0-9-]+$/),
    ]],
    nome: ['', [Validators.required, Validators.maxLength(200)]],
    emailTitular: ['', [Validators.required, Validators.email]],
    cpfTitular: ['', [
      Validators.required,
      Validators.minLength(11),
      Validators.maxLength(11),
      Validators.pattern(/^\d+$/),
    ]],
    senhaTitular: ['', [Validators.required, Validators.minLength(8)]],
  });

  async onSubmit(): Promise<void> {
    if (this.form.invalid) {
      this.form.markAllAsTouched();
      return;
    }

    this.enviando.set(true);

    const request: CreateTenantRequest = {
      slug: this.form.value.slug,
      nome: this.form.value.nome,
      emailTitular: this.form.value.emailTitular,
      cpfTitular: this.form.value.cpfTitular,
      senhaTitular: this.form.value.senhaTitular,
    };

    this.tenantService.criar(request).subscribe({
      next: (response) => {
        this.snackBar.open('Escritório cadastrado com sucesso!', 'Fechar', { duration: 3000 });
        this.router.navigate(['/admin/tenants']);
      },
      error: (error) => {
        const message = error.error?.error?.message || 'Erro ao cadastrar escritório';
        this.snackBar.open(message, 'Fechar', { duration: 5000 });
      },
      complete: () => {
        this.enviando.set(false);
      },
    });
  }
}